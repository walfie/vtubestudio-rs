use crate::data::*;
use crate::error::{Error, Result};

use futures_sink::Sink;
use futures_util::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pin_project! {
    pub struct WebSocketTransport<T = WebSocketStream<MaybeTlsStream<TcpStream>>> {
        #[pin]
        inner: T
    }
}

impl<T> WebSocketTransport<T>
where
    T: Sink<Message, Error = tungstenite::Error>
        + Stream<Item = Result<Message, tungstenite::Error>>,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Sink<RequestEnvelope> for WebSocketTransport<T>
where
    T: Sink<Message, Error = tungstenite::Error>,
{
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_ready(cx)
            .map_err(Error::WebSocket)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        let json = serde_json::to_string(&item).map_err(Error::Json)?;
        self.as_mut()
            .project()
            .inner
            .start_send(Message::Text(json))
            .map_err(Error::WebSocket)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_flush(cx)
            .map_err(Error::WebSocket)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_close(cx)
            .map_err(Error::WebSocket)
    }
}

impl<T> Stream for WebSocketTransport<T>
where
    T: Stream<Item = Result<Message, tungstenite::Error>>,
{
    type Item = Result<ResponseEnvelope, Error>;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            match futures_util::ready!(this.inner.as_mut().poll_next(cx)) {
                Some(Ok(msg)) => match msg {
                    Message::Ping(..) => (),
                    Message::Text(s) => break Some(serde_json::from_str(&s).map_err(Error::Json)),
                    other => break Some(Err(Error::UnexpectedWebSocketMessage(other))),
                },
                Some(Err(e)) => break Some(Err(Error::WebSocket(e))),
                None => break None,
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
