use crate::data::{RequestEnvelope, RequestId, ResponseEnvelope};
use crate::error::{BoxError, Error};
use crate::transport::buffered::BufferedApiTransport;
use crate::transport::event::{EventStream, EventlessApiTransport};

use futures_core::TryStream;
use futures_sink::Sink;
use std::fmt::Write;
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
pub struct IdTagger {
    next: usize,
    buffer: String,
}

impl TagStore<RequestEnvelope, ResponseEnvelope> for IdTagger {
    type Tag = RequestId;

    fn assign_tag(mut self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        // If request already has an ID, use it. Otherwise generate a new one.
        if let Some(id) = &request.request_id {
            return id.clone();
        }

        let id = self.next;
        if write!(self.buffer, "{}", id).is_err() {
            // We don't expect this to happen, but recover just in case
            self.buffer = id.to_string();
        }

        let id = RequestId::from(self.buffer.as_str());
        request.request_id = Some(id.clone());

        self.next += 1;
        self.buffer.clear();
        id
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}

type ServiceInner<T> = MultiplexClient<
    MultiplexTransport<BufferedApiTransport<EventlessApiTransport<T>>, IdTagger>,
    Error,
    RequestEnvelope,
>;

/// A [`Service`] that assigns request IDs to [`RequestEnvelope`]s and matches them to incoming
/// [`ResponseEnvelope`]s.
///
/// This uses [`tokio_tower::multiplex`] to wrap an underlying transport.
#[derive(Debug)]
pub struct ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope>,
{
    service: ServiceInner<T>,
}

impl<T> ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream<Ok = ResponseEnvelope> + Send + 'static,
    <T as TryStream>::Error: Send,
    BoxError: From<<T as Sink<RequestEnvelope>>::Error>,
    BoxError: From<<T as TryStream>::Error>,
{
    /// Create a new [`ApiService`] and corresponding [`EventStream`].
    pub fn new(transport: T, buffer_size: usize) -> (Self, EventStream<T>) {
        Self::with_error_handler(
            transport,
            buffer_size,
            |error| tracing::error!(%error, "Transport error"),
        )
    }

    /// Create a new [`ApiService`] with an internal handler for transport errors.
    pub fn with_error_handler<F>(
        transport: T,
        buffer_size: usize,
        on_service_error: F,
    ) -> (Self, EventStream<T>)
    where
        F: FnOnce(Error) + Send + 'static,
    {
        let tagger = IdTagger {
            next: 0,
            buffer: String::new(),
        };

        let (eventless_transport, event_stream) = EventlessApiTransport::new(transport);
        let buffered_transport = BufferedApiTransport::new(eventless_transport, buffer_size);

        let multiplex_transport = MultiplexTransport::new(buffered_transport, tagger);
        let service = MultiplexClient::with_error_handler(multiplex_transport, on_service_error);

        (Self { service }, event_stream)
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
