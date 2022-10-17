use crate::data::{EventData, RequestEnvelope, ResponseEnvelope};
use crate::error::BoxError;
use crate::service::api::ApiService;

use futures_util::future::MapOk;
use futures_util::{Sink, TryFutureExt};
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
pub struct MakeApiService<M, S, R> {
    maker: M,
    event_sink: S,
    _req: PhantomData<fn(R)>,
}

impl<M, S, R> MakeApiService<M, S, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope>,
    S: Sink<EventData, Error = Error> + Send + Unpin + 'static,
{
    /// Creates a new [`MakeApiService`].
    pub fn new(maker: M, event_sink: S) -> Self {
        Self {
            maker,
            event_sink,
            _req: PhantomData,
        }
    }
}

impl<M, S, R> MakeApiService<M, S, R> {
    /// Consumes `self`, returning the inner service and event sink.
    pub fn into_inner(self) -> (M, S) {
        (self.maker, self.event_sink)
    }
}

impl<M, S, R> Service<R> for MakeApiService<M, S, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope> + Send,
    M::Future: Send + 'static,
    M::Transport: Send + 'static,
    S: Sink<EventData, Error = Error> + Send + Unpin + 'static,
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
        self.maker
            .make_transport(request)
            .map_ok(|transport| ApiService::new(transport, self.event_sink))
    }
}

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]

    use crate::transport::TungsteniteApiTransport;
    use futures_util::FutureExt;
    use std::future::Future;
    use std::pin::Pin;
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

    impl<S, R> MakeApiService<TungsteniteConnector, S, R>
    where
        S: Sink<EventData, Error = Error> + Send + Unpin + 'static,
        R: Send + IntoClientRequest + Unpin + 'static,
    {
        /// Creates a new [`MakeApiService`] using [`tokio_tungstenite`] as the underlying transport.
        pub fn new_tungstenite(event_sink: S) -> Self {
            MakeApiService::new(TungsteniteConnector, event_sink)
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
