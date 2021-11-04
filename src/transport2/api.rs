use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::TransportError;
use crate::transport2::convert::MessageConvert;

use futures_core::{Stream, TryStream};
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    #[derive(Debug, Clone)]
    pub struct ApiTransport<T, M> {
        #[pin]
        transport: T,
        converter: M
    }
}

impl<T, M> ApiTransport<T, M>
where
    T: Sink<M::Message> + TryStream,
    M: MessageConvert,
{
    pub fn new(transport: T, converter: M) -> Self {
        Self {
            transport,
            converter,
        }
    }
}

impl<T, M> Sink<RequestEnvelope> for ApiTransport<T, M>
where
    T: Sink<M::Message>,
    M: MessageConvert,
{
    type Error = TransportError<<T as Sink<M::Message>>::Error>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .transport
            .poll_ready(cx)
            .map_err(TransportError::Underlying)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        let json = serde_json::to_string(&item).map_err(TransportError::Json)?;
        self.as_mut()
            .project()
            .transport
            .start_send(M::create_message(json))
            .map_err(TransportError::Underlying)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .transport
            .poll_flush(cx)
            .map_err(TransportError::Underlying)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .transport
            .poll_close(cx)
            .map_err(TransportError::Underlying)
    }
}

impl<T, M> Stream for ApiTransport<T, M>
where
    T: TryStream<Ok = M::Message>,
    M: MessageConvert,
{
    type Item = Result<ResponseEnvelope, TransportError<<T as TryStream>::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            match futures_util::ready!(this.transport.as_mut().try_poll_next(cx)) {
                Some(Ok(msg)) => {
                    if let Some(s) = M::extract_text(msg) {
                        let json = serde_json::from_str(&s).map_err(TransportError::Json);
                        break Some(json);
                    }
                }
                Some(Err(e)) => break Some(Err(TransportError::Underlying(e))),
                None => break None,
            }
        })
    }
}
