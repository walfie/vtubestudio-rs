use crate::client::Client;
use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::Error;

use futures_core::TryStream;
use futures_sink::Sink;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tower::multiplex::{Client as MultiplexClient, MultiplexTransport, TagStore};
use tower::Service;

#[derive(Debug)]
pub struct IdTagger(usize);

impl TagStore<RequestEnvelope, ResponseEnvelope> for IdTagger {
    type Tag = String;

    fn assign_tag(mut self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        let id = self.0.to_string();
        request.request_id = Some(id.clone());
        self.0 += 1;
        id
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}

type ServiceInner<T> = MultiplexClient<
    MultiplexTransport<T, IdTagger>,
    Error<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>,
    RequestEnvelope,
>;

#[derive(Debug)]
pub struct ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream,
{
    client: ServiceInner<T>,
}

impl<T> ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    <T as Sink<RequestEnvelope>>::Error: Send,
    <T as TryStream>::Error: Send,
{
    pub fn new(transport: T) -> Self {
        Self::with_error_handler(transport, |_| ())
    }

    pub fn with_error_handler<F>(transport: T, on_service_error: F) -> Self
    where
        F: FnOnce(Error<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>)
            + Send
            + 'static,
    {
        let tagger = IdTagger(0);

        let multiplex_transport = MultiplexTransport::new(transport, tagger);
        let client = MultiplexClient::with_error_handler(multiplex_transport, on_service_error);

        Self { client }
    }

    pub fn to_client(self) -> Client<Self> {
        Client::new(self)
    }
}

impl<T> Service<RequestEnvelope> for ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    <T as Sink<RequestEnvelope>>::Error: Send,
    <T as TryStream>::Error: Send,
{
    type Response = ResponseEnvelope;
    type Error = Error<<T as TryStream>::Error, <T as Sink<RequestEnvelope>>::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.client.poll_ready(cx)
    }

    fn call(&mut self, req: RequestEnvelope) -> Self::Future {
        self.client.call(req)
    }
}
