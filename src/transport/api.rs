use crate::codec::{MessageCodec, TungsteniteCodec};
use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::BoxError;

use futures_core::{Stream, TryStream};
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tungstenite::tungstenite;

pin_project! {
    #[derive(Debug, Clone)]
    pub struct ApiTransport<T, C> {
        #[pin]
        transport: T,
        codec: C
    }
}

impl<T, C> ApiTransport<T, C>
where
    T: Sink<C::Message> + TryStream,
    C: MessageCodec,
{
    pub fn new(transport: T, codec: C) -> Self {
        Self { transport, codec }
    }
}

impl<T> ApiTransport<T, TungsteniteCodec>
where
    T: Sink<tungstenite::Message> + TryStream,
{
    pub fn new_tungstenite(transport: T) -> Self {
        ApiTransport::new(transport, TungsteniteCodec)
    }
}

impl<T, C> Sink<RequestEnvelope> for ApiTransport<T, C>
where
    T: Sink<C::Message>,
    C: MessageCodec,
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
        let json_str = serde_json::to_string(&item).map_err(Box::new)?;
        self.as_mut()
            .project()
            .transport
            .start_send(C::encode(json_str))
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

impl<T, C> Stream for ApiTransport<T, C>
where
    T: TryStream<Ok = C::Message>,
    C: MessageCodec,
    BoxError: From<T::Error>,
{
    type Item = Result<ResponseEnvelope, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            match futures_util::ready!(this.transport.as_mut().try_poll_next(cx)) {
                Some(Ok(msg)) => {
                    if let Some(s) = C::decode(msg) {
                        break Some(serde_json::from_str(&s).map_err(|e| Box::new(e) as BoxError));
                    }
                }
                Some(Err(e)) => break Some(Err(e.into())),
                None => break None,
            }
        })
    }
}
