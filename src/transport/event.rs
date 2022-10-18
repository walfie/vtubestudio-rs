use crate::data::{EventData, RequestEnvelope, ResponseEnvelope};
use crate::error::{BoxError, Error};

use futures_core::Stream;
use futures_sink::Sink;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use pin_project_lite::pin_project;
use split_stream_by::{Either, LeftSplitByMap, RightSplitByMap, SplitStreamByMapExt};
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// An API transport that excludes [`Event`] responses from the stream.
    ///
    /// [`Event`]s can be retrieved from the corresponding [`EventStream`].
    pub struct EventlessApiTransport<T> {
        #[pin]
        sink: SplitSink<T, RequestEnvelope>,
        #[pin]
        stream: RightSplitByMap<
            Result<ResponseEnvelope, Error>,
            Result<EventData, Error>,
            Result<ResponseEnvelope, Error>,
            SplitStream<T>,
            SplitFn,
        >,
    }
}

pin_project! {
    /// A stream of [`Event`]s. Created by [`EventlessApiTransport::new`].
    pub struct EventStream<T> {
        #[pin]
        events: LeftSplitByMap<
            Result<ResponseEnvelope, Error>,
            Result<EventData, Error>,
            Result<ResponseEnvelope, Error>,
            SplitStream<T>,
            SplitFn,
        >,
    }
}

impl<T> fmt::Debug for EventlessApiTransport<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventTransport")
            .field("sink", &self.sink)
            .field("stream", &"Stream") // RightSplitByMap doesn't impl Debug
            .finish()
    }
}

type SplitFn = fn(
    Result<ResponseEnvelope, Error>,
) -> Either<Result<EventData, Error>, Result<ResponseEnvelope, Error>>;

impl<T> EventlessApiTransport<T>
where
    T: Sink<RequestEnvelope>
        + Stream<Item = Result<ResponseEnvelope, Error>>
        + Unpin
        + Send
        + 'static,
{
    /// Creates a new [`ApiTransport`].
    pub fn new<S>(transport: T) -> (Self, EventStream<T>) {
        let (resp_sink, resp_stream) = transport.split();

        let (events, responses) = resp_stream.split_by_map(
            (|resp| match resp {
                Ok(r) if r.message_type().is_event() => Either::Left(r.parse_event()),
                other => Either::Right(other),
            }) as SplitFn,
        );

        let event_stream = EventStream { events };
        let transport_out = Self {
            sink: resp_sink,
            stream: responses,
        };

        (transport_out, event_stream)
    }
}

impl<T> Sink<RequestEnvelope> for EventlessApiTransport<T>
where
    T: Sink<RequestEnvelope>,
    BoxError: From<T::Error>,
{
    type Error = BoxError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .sink
            .poll_ready(cx)
            .map_err(BoxError::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        self.as_mut()
            .project()
            .sink
            .start_send(item)
            .map_err(BoxError::from)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .sink
            .poll_flush(cx)
            .map_err(BoxError::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .sink
            .poll_close(cx)
            .map_err(BoxError::from)
    }
}

impl<T> Stream for EventlessApiTransport<T>
where
    T: Stream<Item = Result<ResponseEnvelope, Error>>,
{
    type Item = Result<ResponseEnvelope, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}

impl<T> Stream for EventStream<T>
where
    T: Stream<Item = Result<ResponseEnvelope, Error>>,
{
    type Item = Result<EventData, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().events.poll_next(cx)
    }
}
