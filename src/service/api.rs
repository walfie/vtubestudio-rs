use crate::data::{RequestEnvelope, RequestId, ResponseEnvelope};
use crate::error::{BoxError, Error};

use futures_core::TryStream;
use futures_sink::Sink;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tower::multiplex::{Client as MultiplexClient, MultiplexTransport, TagStore};
use tower::Service;

crate::cfg_feature! {
    #![feature = "tokio-tungstenite"]
    use crate::transport::TungsteniteApiTransport;

    /// Type alias for an [`ApiService`] wrapping a [`TungsteniteApiTransport`].
    pub type TungsteniteApiService = ApiService<TungsteniteApiTransport>;
}

/// Struct describing how to tag [`RequestEnvelope`]s and extract tags from [`ResponseEnvelope`]s.
#[derive(Debug)]
pub struct IdTagger(usize);

impl TagStore<RequestEnvelope, ResponseEnvelope> for IdTagger {
    type Tag = RequestId;

    fn assign_tag(mut self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        let id = RequestId::from(self.0.to_string());
        request.request_id = Some(id.clone());
        self.0 += 1;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}

type ServiceInner<T> = MultiplexClient<MultiplexTransport<T, IdTagger>, Error, RequestEnvelope>;

/// A [`Service`] that assigns request IDs to [`RequestEnvelope`]s and matches them to incoming
/// [`ResponseEnvelope`]s.
///
/// This uses [`tokio_tower::multiplex`] to wrap an underlying transport.
#[derive(Debug)]
pub struct ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream,
{
    service: ServiceInner<T>,
}

impl<T> ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    BoxError: From<<T as Sink<RequestEnvelope>>::Error>,
    BoxError: From<<T as TryStream>::Error>,
{
    /// Create a new [`ApiService`].
    pub fn new(transport: T) -> Self {
        Self::with_error_handler(transport, |_| ())
    }

    /// Create a new [`ApiService`] with an error handler.
    pub fn with_error_handler<F>(transport: T, on_service_error: F) -> Self
    where
        F: FnOnce(Error) + Send + 'static,
    {
        let tagger = IdTagger(0);

        let multiplex_transport = MultiplexTransport::new(transport, tagger);
        let service = MultiplexClient::with_error_handler(multiplex_transport, on_service_error);

        Self { service }
    }
}

impl<T> Service<RequestEnvelope> for ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + 'static,
    BoxError: From<<T as Sink<RequestEnvelope>>::Error>,
    BoxError: From<<T as TryStream>::Error>,
{
    type Response = ResponseEnvelope;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: RequestEnvelope) -> Self::Future {
        self.service.call(req)
    }
}
