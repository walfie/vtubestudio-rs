use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::BoxError;
use crate::service::api::ApiService;

use futures_util::future::MapOk;
use futures_util::TryFutureExt;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tokio_tower::MakeTransport;
use tower::Service;

/// A [`Service`] that yields new [`ApiService`]s.
///
/// This wraps a [`MakeTransport`] (such as [`TungsteniteConnector`]), describing how to connect to
/// a websocket sink/stream. This is used for as the inner service for the
/// [`Reconnect`](tower::reconnect::Reconnect) middleware.
#[derive(Clone, Debug)]
pub struct MakeApiService<M, R> {
    maker: M,
    _req: PhantomData<fn(R)>,
}

impl<M, R> MakeApiService<M, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope>,
{
    /// Creates a new [`MakeApiService`].
    pub fn new(maker: M) -> Self {
        Self {
            maker,
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
    BoxError: From<M::Error>,
    BoxError: From<M::SinkError>,
{
    type Response = ApiService<M::Transport>;
    type Error = M::MakeError;
    type Future = MapOk<M::Future, fn(M::Transport) -> Self::Response>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.maker.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        self.maker.make_transport(request).map_ok(ApiService::new)
    }
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    use crate::transport::connector::TungsteniteConnector;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    impl<R> MakeApiService<TungsteniteConnector, R>
    where
        R: Send + IntoClientRequest + Unpin + 'static,
    {
        /// Creates a new [`MakeApiService`] using [`tokio_tungstenite`] as the underlying transport.
        pub fn new_tungstenite() -> Self {
            MakeApiService::new(TungsteniteConnector)
        }
    }
}
