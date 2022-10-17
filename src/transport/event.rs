use crate::data::{EventData, RequestEnvelope, ResponseEnvelope};
use crate::error::BoxError;

use futures_core::{Stream, TryStream};
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// A transport that forwards events to a separate sink.
    #[derive(Debug, Clone)]
    pub(crate) struct EventTransport<T> {
        #[pin]
        transport: T,
    }
}

impl<T> EventTransport<T>
where
    T: Sink<RequestEnvelope> + TryStream,
{
    /// Creates a new [`ApiTransport`].
    pub fn new<S>(transport: T, event_sink: S) -> Self
    where
        S: Sink<EventData>,
    {
        // TODO: Do something with event_sink
        Self { transport }
    }
}

impl<T> Sink<RequestEnvelope> for EventTransport<T>
where
    T: Sink<RequestEnvelope>,
    BoxError: From<T::Error>,
{
    type Error = BoxError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .transport
            .poll_ready(cx)
            .map_err(BoxError::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        self.as_mut()
            .project()
            .transport
            .start_send(item)
            .map_err(BoxError::from)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .transport
            .poll_flush(cx)
            .map_err(BoxError::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .transport
            .poll_close(cx)
            .map_err(BoxError::from)
    }
}

impl<T> Stream for EventTransport<T>
where
    T: TryStream<Ok = ResponseEnvelope>,
    T::Error: Into<BoxError>,
{
    type Item = Result<ResponseEnvelope, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project()
            .transport
            .try_poll_next(cx)
            .map_err(Into::into)
    }
}
