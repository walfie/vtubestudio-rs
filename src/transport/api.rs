use crate::codec::MessageCodec;
use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::BoxError;

use futures_core::{Stream, TryStream};
use futures_sink::Sink;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    use tokio_tungstenite::tungstenite;
    use crate::codec::TungsteniteCodec;

    impl<T> ApiTransport<T, TungsteniteCodec>
    where
        T: Sink<tungstenite::Message> + TryStream,
    {
        /// Creates a new [`ApiTransport`] for sending/receiving [`tokio_tungstenite`] messages.
        pub fn new_tungstenite(transport: T) -> Self {
            ApiTransport::new(transport, TungsteniteCodec)
        }
    }
}

pin_project! {
    /// A transport that uses a [`MessageCodec`] to implement:
    ///
    /// * [`Sink`] for accepting [`RequestEnvelope`] messages and converting them into websocket
    ///   text messages
    /// * [`TryStream`] for receiving websocket messages and converting them to
    ///   [`ResponseEnvelope`] messages
    ///
    /// This is a layer of abstraction to allow this library to be compatible with multiple
    /// websocket libraries.
    #[derive(Debug, Clone)]
    pub struct ApiTransport<T, C> {
        #[pin]
        transport: T,
        codec: C
    }
}

impl<T, C> ApiTransport<T, C>
where
    T: Sink<C::Output> + TryStream,
    C: MessageCodec,
{
    /// Creates a new [`ApiTransport`].
    pub fn new(transport: T, codec: C) -> Self {
        Self { transport, codec }
    }
}

impl<T, C> ApiTransport<T, C> {
    /// Consumes `self`, returning the inner transport.
    pub fn into_inner(self) -> T {
        self.transport
    }
}

impl<T, C> Sink<RequestEnvelope> for ApiTransport<T, C>
where
    T: Sink<C::Output>,
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
    T: TryStream<Ok = C::Input>,
    T::Error: Into<BoxError>,
    C: MessageCodec,
    C::Error: Into<BoxError>,
{
    type Item = Result<ResponseEnvelope, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            match futures_util::ready!(this.transport.as_mut().try_poll_next(cx)) {
                Some(Ok(msg)) => {
                    if let Some(s) = C::decode(msg).map_err(Into::into)? {
                        break Some(serde_json::from_str(&s).map_err(Into::into));
                    }
                }
                Some(Err(e)) => break Some(Err(e.into())),
                None => break None,
            }
        })
    }
}
