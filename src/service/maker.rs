use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::BoxError;
use crate::service::api::ApiService;
use crate::transport::EventStream;

use futures_util::TryFutureExt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tower::MakeTransport;
use tower::Service;

/// A [`Service`] that yields new [`ApiService`]s and [`EventStream`]s.
///
/// This wraps a [`MakeTransport`] (such as [`TungsteniteConnector`]), describing how to connect to
/// a websocket sink/stream. This is used for as the inner service for the
/// [`Reconnect`](tower::reconnect::Reconnect) middleware.
#[derive(Clone, Debug)]
pub struct MakeApiService<M, R> {
    maker: M,
    buffer_size: usize,
    _req: PhantomData<fn(R)>,
}

impl<M, R> MakeApiService<M, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope>,
{
    /// Creates a new [`MakeApiService`].
    pub fn new(maker: M, buffer_size: usize) -> Self {
        Self {
            maker,
            buffer_size,
            _req: PhantomData,
        }
    }
}

impl<M, R> MakeApiService<M, R> {
    /// Consumes `self`, returning the inner service.
    pub fn into_inner(self) -> M {
        self.maker
    }
}

impl<M, R> Service<R> for MakeApiService<M, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope> + Send,
    M::Future: Send + 'static,
    M::Transport: Send + 'static,
    M::Error: Send,
    BoxError: From<M::Error> + From<M::SinkError>,
{
    type Response = (ApiService<M::Transport>, EventStream<M::Transport>);
    type Error = M::MakeError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.maker.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        let buffer_size = self.buffer_size;
        Box::pin(
            self.maker
                .make_transport(request)
                .map_ok(move |transport| ApiService::new(transport, buffer_size)),
        )
    }
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]

    use crate::transport::TungsteniteApiTransport;
    use futures_util::FutureExt;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    /// A [`Service`] for creating new [`TungsteniteApiTransport`]s.
    ///
    /// This is used by [`tower::reconnect::Reconnect`] (used in
    /// [`ClientBuilder`](crate::ClientBuilder)) for lazily connecting/reconnecting to websockets.
    #[derive(Debug, Clone)]
    pub struct TungsteniteConnector;
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    use crate::{Error, ErrorKind};

    impl<R> MakeApiService<TungsteniteConnector, R>
    where
        R: Send + IntoClientRequest + Unpin + 'static,
    {
        /// Creates a new [`MakeApiService`] using [`tokio_tungstenite`] as the underlying transport.
        pub fn new_tungstenite(buffer_size: usize) -> Self {
            MakeApiService::new(TungsteniteConnector, buffer_size)
        }
    }

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
            let transport = tokio_tungstenite::connect_async(request).map(|result| match result {
                Ok((transport, _resp)) => Ok(TungsteniteApiTransport::new_tungstenite(transport)),
                Err(e) => Err(Error::new(ErrorKind::ConnectionRefused).with_source(e)),
            });
            Box::pin(transport)
        }
    }
}
