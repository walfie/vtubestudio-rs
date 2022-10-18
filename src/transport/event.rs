use crate::data::{EventData, RequestEnvelope, ResponseEnvelope};
use crate::error::{BoxError, Error};

use futures_core::{Stream, TryStream};
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
    pub struct EventlessApiTransport<T, E> {
        #[pin]
        sink: SplitSink<T, RequestEnvelope>,
        #[pin]
        stream: RightSplitByMap<
            Result<ResponseEnvelope, E>,
            Result<EventData, Error>,
            Result<ResponseEnvelope, E>,
            SplitStream<T>,
            SplitFn<E>,
        >,
    }
}

pin_project! {
    /// A stream of [`Event`]s. Created by [`EventlessApiTransport::new`].
    pub struct EventStream<T, E> {
        #[pin]
        events: LeftSplitByMap<
            Result<ResponseEnvelope, E>,
            Result<EventData, Error>,
            Result<ResponseEnvelope, E>,
            SplitStream<T>,
            SplitFn<E>,
        >,
    }
}

impl<T, E> fmt::Debug for EventlessApiTransport<T, E>
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

type SplitFn<E> = fn(
    Result<ResponseEnvelope, E>,
) -> Either<Result<EventData, Error>, Result<ResponseEnvelope, E>>;

impl<T, E> EventlessApiTransport<T, E>
where
    T: Sink<RequestEnvelope> + Stream<Item = Result<ResponseEnvelope, E>> + Unpin + Send + 'static,
    E: Send + 'static,
{
    /// Creates a new [`ApiTransport`].
    pub fn new<S>(transport: T) -> (Self, EventStream<T, E>) {
        let (resp_sink, resp_stream) = transport.split();

        let (events, responses) = resp_stream.split_by_map(
            (|resp| match resp {
                Ok(r) if r.message_type().is_event() => Either::Left(r.parse_event()),
                other => Either::Right(other),
            }) as SplitFn<E>,
        );

        let event_stream = EventStream { events };
        let transport_out = Self {
            sink: resp_sink,
            stream: responses,
        };

        (transport_out, event_stream)
    }
}

impl<T, E> Sink<RequestEnvelope> for EventlessApiTransport<T, E>
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

impl<T, E> Stream for EventlessApiTransport<T, E>
where
    T: Stream<Item = Result<ResponseEnvelope, E>>,
    E: Into<BoxError>,
{
    type Item = Result<ResponseEnvelope, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.try_poll_next(cx).map_err(Into::into)
    }
}

impl<T, E> Stream for EventStream<T, E>
where
    T: Stream<Item = Result<ResponseEnvelope, E>>,
{
    type Item = Result<EventData, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().events.poll_next(cx)
    }
}
