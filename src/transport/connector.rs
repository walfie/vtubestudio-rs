crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    pub use self::tungstenite::TungsteniteConnector;
}

#[cfg(feature = "tokio-tungstenite")]
mod tungstenite {
    use crate::{Error, ErrorKind};

    use futures_util::TryFutureExt;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tower::Service;

    use crate::transport::TungsteniteApiTransport;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    /// A [`Service`] for creating new [`TungsteniteApiTransport`]s.
    ///
    /// This is used by [`tower::reconnect::Reconnect`] (used in
    /// [`ClientBuilder`](crate::ClientBuilder)) for lazily connecting/reconnecting to websockets.
    #[derive(Debug, Clone)]
    pub struct TungsteniteConnector;

    impl<R> Service<R> for TungsteniteConnector
    where
        R: IntoClientRequest + Unpin + Send + 'static,
    {
        type Response = TungsteniteApiTransport;
        type Error = Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: R) -> Self::Future {
            let transport = tokio_tungstenite::connect_async(request)
                .map_err(|e| Error::new(ErrorKind::ConnectionRefused).with_source(e))
                .map_ok(|(transport, _resp)| TungsteniteApiTransport::new_tungstenite(transport));
            Box::pin(transport)
        }
    }
}
