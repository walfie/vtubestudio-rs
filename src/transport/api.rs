use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::TransportError;
use crate::transport::WebSocketTransport;

use futures_core::Stream;
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    #[derive(Debug)]
    pub struct ApiTransport<T: WebSocketTransport> {
        #[pin]
        inner: T::Underlying
    }
}

impl<T> ApiTransport<T>
where
    T: WebSocketTransport,
{
    pub fn new(inner: T::Underlying) -> Self {
        Self { inner }
    }
}

impl<T> Sink<RequestEnvelope> for ApiTransport<T>
where
    T: WebSocketTransport,
{
    type Error = TransportError<T>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_ready(cx)
            .map_err(TransportError::Write)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        let json = serde_json::to_string(&item).map_err(TransportError::Json)?;
        self.as_mut()
            .project()
            .inner
            .start_send(T::create_message(json))
            .map_err(TransportError::Write)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_flush(cx)
            .map_err(TransportError::Write)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_close(cx)
            .map_err(TransportError::Write)
    }
}

impl<T> Stream for ApiTransport<T>
where
    T: WebSocketTransport,
{
    type Item = Result<ResponseEnvelope, TransportError<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            match futures_util::ready!(this.inner.as_mut().poll_next(cx)) {
                Some(Ok(msg)) => {
                    if let Some(s) = T::extract_text(msg) {
                        let json = serde_json::from_str(&s).map_err(TransportError::Json);
                        break Some(json);
                    }
                }
                Some(Err(e)) => break Some(Err(TransportError::Read(e))),
                None => break None,
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
