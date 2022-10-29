use crate::data::{Event, RequestEnvelope, ResponseEnvelope};
use crate::error::Error;

use futures_core::{Stream, TryStream};
use futures_sink::Sink;
use futures_util::stream::{IntoStream, SplitSink, SplitStream};
use futures_util::{StreamExt, TryStreamExt};
use pin_project_lite::pin_project;
use split_stream_by::{
    Either, LeftSplitByMapBuffered, RightSplitByMapBuffered, SplitStreamByMapExt,
};
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

const BUF_SIZE: usize = 64;

pin_project! {
    /// An API transport that excludes [`Event`] responses from the stream.
    ///
    /// [`Event`]s can be retrieved from the corresponding [`EventStream`].
    pub(crate) struct EventlessApiTransport<T> where T: TryStream {
        #[pin]
        sink: SplitSink<IntoStream<T>, RequestEnvelope>,
        #[pin]
        stream: RightSplitByMapBuffered<
            Result<ResponseEnvelope, T::Error>,
            Result<Event, Error>,
            Result<ResponseEnvelope, T::Error>,
            SplitStream<IntoStream<T>>,
            SplitFn<T::Error>,
            BUF_SIZE,
        >,
    }
}

pin_project! {
    /// A stream of events.
    ///
    /// This is created when constructing an [`ApiService`](crate::service::ApiService).
    pub struct EventStream<S> where S: TryStream {
        #[pin]
        events: LeftSplitByMapBuffered<
            Result<ResponseEnvelope, S::Error>,
            Result<Event, Error>,
            Result<ResponseEnvelope, S::Error>,
            SplitStream<IntoStream<S>>,
            SplitFn<S::Error>,
            BUF_SIZE,
        >,
    }
}

impl<T> fmt::Debug for EventlessApiTransport<T>
where
    T: TryStream + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventTransport")
            .field("sink", &self.sink)
            .field("stream", &"Stream") // RightSplitByMapBuffered doesn't impl Debug
            .finish()
    }
}

type SplitFn<E> =
    fn(Result<ResponseEnvelope, E>) -> Either<Result<Event, Error>, Result<ResponseEnvelope, E>>;

impl<T> EventlessApiTransport<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope>,
{
    /// Creates a new [`ApiTransport`].
    pub fn new(transport: T) -> (Self, EventStream<T>) {
        let (resp_sink, resp_stream) = transport.into_stream().split();

        let (events, responses) = resp_stream.split_by_map_buffered::<BUF_SIZE>(
            (|resp| match resp {
                Ok(r) if r.message_type().is_event() => Either::Left(r.parse_event()),
                other => Either::Right(other),
            }) as SplitFn<<T as TryStream>::Error>,
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
    T: Sink<RequestEnvelope> + TryStream,
{
    type Error = <T as Sink<RequestEnvelope>>::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        self.as_mut().project().sink.start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_close(cx)
    }
}

impl<T> Stream for EventlessApiTransport<T>
where
    T: TryStream<Ok = ResponseEnvelope>,
{
    type Item = Result<ResponseEnvelope, T::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.try_poll_next(cx).map_err(Into::into)
    }
}

impl<T> Stream for EventStream<T>
where
    T: TryStream<Ok = ResponseEnvelope>,
{
    type Item = Result<Event, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().events.try_poll_next(cx)
    }
}
