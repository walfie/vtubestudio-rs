use crate::transport::TungsteniteApiTransport;
use crate::{ServiceError, ServiceErrorKind};

use futures_util::TryFutureExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tower::Service;

#[derive(Debug, Clone)]
pub struct TungsteniteConnector;

impl<R> Service<R> for TungsteniteConnector
where
    R: IntoClientRequest + Unpin + Send + 'static,
{
    type Response = TungsteniteApiTransport;
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: R) -> Self::Future {
        let transport = tokio_tungstenite::connect_async(request)
            .map_err(|e| ServiceError::new(ServiceErrorKind::ConnectionRefused).with_source(e))
            .map_ok(|(transport, _resp)| TungsteniteApiTransport::new_tungstenite(transport));
        Box::pin(transport)
    }
}
