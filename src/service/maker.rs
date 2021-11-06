use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::service::api::ApiService;

use futures_util::future::MapOk;
use futures_util::TryFutureExt;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tokio_tower::MakeTransport;
use tower::Service;

#[derive(Debug)]
pub struct MakeApiService<M, R> {
    maker: M,
    _req: PhantomData<fn(R)>,
}

impl<M, R> MakeApiService<M, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope>,
{
    pub fn new(maker: M) -> Self {
        Self {
            maker,
            _req: PhantomData,
        }
    }
}

impl<M, R> Service<R> for MakeApiService<M, R>
where
    M: MakeTransport<R, RequestEnvelope, Item = ResponseEnvelope> + Send,
    M::Future: Send + 'static,
    M::Transport: Send + 'static,
    M::Error: Send,
    M::SinkError: Send,
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
