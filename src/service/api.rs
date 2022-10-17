use crate::data::{EventData, RequestEnvelope, RequestId, ResponseEnvelope};
use crate::error::{BoxError, Error};

use futures_core::{Stream, TryStream};
use futures_util::stream::SplitSink;
use futures_util::{Sink, StreamExt};
use pin_project_lite::pin_project;
use split_stream_by::{Either, SplitStreamByMapExt};
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
    service: ServiceInner<
        SplitSinkStream<
            SplitSink<T, RequestEnvelope>,
            Box<
                dyn Stream<Item = Result<ResponseEnvelope, <T as TryStream>::Error>>
                    + Unpin
                    + Send
                    + 'static,
            >,
            /*
            RightSplitByMap<
                Result<ResponseEnvelope, <T as TryStream>::Error>,
                EventData,
                Result<ResponseEnvelope, <T as TryStream>::Error>,
                SplitStream<T>,
                fn(
                    Result<ResponseEnvelope, <T as TryStream>::Error>,
                )
                    -> Either<EventData, Result<ResponseEnvelope, <T as TryStream>::Error>>,
            >,
                */
        >,
    >,
}

impl<T, E> ApiService<T>
where
    T: Sink<RequestEnvelope> + Stream<Item = Result<ResponseEnvelope, E>> + Send + 'static,
    E: Send + 'static,
    BoxError: From<<T as Sink<RequestEnvelope>>::Error>,
    BoxError: From<E>,
{
    /// Create a new [`ApiService`].
    pub fn new<S>(transport: T, event_sink: S) -> Self
    where
        S: Sink<EventData, Error = Error> + Send + Unpin + 'static,
    {
        Self::with_error_handler(transport, event_sink, |_| ())
    }

    /// Create a new [`ApiService`] with an error handler.
    pub fn with_error_handler<F, S>(transport: T, event_sink: S, on_service_error: F) -> Self
    where
        S: Sink<EventData, Error = Error> + Send + Unpin + 'static,
        F: FnOnce(Error) + Send + 'static,
    {
        let tagger = IdTagger {
            next: 0,
            buffer: String::new(),
        };

        let (resp_sink, resp_stream) = transport.split();

        let (events, responses) = resp_stream.split_by_map(|resp| match resp {
            Ok(r) if r.message_type().is_event() => Either::Left(r.parse_event()),
            other => Either::Right(other),
        });

        tokio::spawn(events.forward(event_sink));

        let multiplex_transport = MultiplexTransport::new(
            SplitSinkStream {
                sink: resp_sink,
                stream: Box::new(responses)
                    as Box<
                        dyn Stream<Item = Result<ResponseEnvelope, <T as TryStream>::Error>>
                            + Unpin
                            + Send
                            + 'static,
                    >,
            },
            tagger,
        );
        let service = MultiplexClient::with_error_handler(multiplex_transport, on_service_error);

        Self { service }
    }
}

impl<T> Service<RequestEnvelope> for ApiService<T>
where
    T: Sink<RequestEnvelope> + TryStream + 'static,
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

pin_project! {
    struct SplitSinkStream<SinkT, StreamT> {
        #[pin]
        sink: SinkT,
        #[pin]
        stream: StreamT,
    }
}

impl<T, SinkT, StreamT> Sink<T> for SplitSinkStream<SinkT, StreamT>
where
    SinkT: Sink<T>,
{
    type Error = SinkT::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.as_mut().project().sink.start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut().project().sink.poll_close(cx)
    }
}

impl<SinkT, StreamT> Stream for SplitSinkStream<SinkT, StreamT>
where
    StreamT: TryStream,
{
    type Item = StreamT::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().project().stream.poll_next(cx)
    }
}
