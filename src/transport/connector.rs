use crate::codec::TungsteniteCodec;
use crate::transport::api::ApiTransport;

use futures_util::TryFutureExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tower::Service;

#[derive(Debug)]
pub struct TungsteniteConnector;

impl<R> Service<R> for TungsteniteConnector
where
    R: IntoClientRequest + Unpin + Send + 'static,
{
    type Response = ApiTransport<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteCodec>;
    type Error = tungstenite::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: R) -> Self::Future {
        let transport = tokio_tungstenite::connect_async(request)
            .map_ok(|(transport, _resp)| ApiTransport::new(transport, TungsteniteCodec));
        Box::pin(transport)
    }
}
