use crate::data::*;
use crate::error::{Error, Result};

use futures_core::stream::TryStream;
use futures_sink::Sink;
use futures_util::Stream;
use pin_project_lite::pin_project;
use std::convert::TryFrom;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_tower::multiplex::{Client as MultiplexClient, TagStore};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tower::util::ServiceExt;
use tower::Service;
use uuid::Uuid;

// TODO: Add ID generator trait
struct Tagger;
impl TagStore<RequestEnvelope, ResponseEnvelope> for Tagger {
    type Tag = String;

    fn assign_tag(self: Pin<&mut Self>, request: &mut RequestEnvelope) -> Self::Tag {
        let uuid = Uuid::new_v4().to_string();
        request.request_id = Some(uuid.clone());
        uuid
    }

    fn finish_tag(self: Pin<&mut Self>, response: &ResponseEnvelope) -> Self::Tag {
        response.request_id.clone()
    }
}

pin_project! {
    pub struct WebSocketTransport<T = WebSocketStream<MaybeTlsStream<TcpStream>>> {
        #[pin]
        inner: T
    }
}

impl<T> WebSocketTransport<T>
where
    T: Sink<Message, Error = tungstenite::Error>
        + Stream<Item = Result<Message, tungstenite::Error>>,
{
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Sink<RequestEnvelope> for WebSocketTransport<T>
where
    T: Sink<Message, Error = tungstenite::Error>,
{
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_ready(cx)
            .map_err(Error::WebSocket)
    }

    fn start_send(mut self: Pin<&mut Self>, item: RequestEnvelope) -> Result<(), Self::Error> {
        let json = serde_json::to_string(&item).map_err(Error::Json)?;
        self.as_mut()
            .project()
            .inner
            .start_send(Message::Text(json))
            .map_err(Error::WebSocket)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_flush(cx)
            .map_err(Error::WebSocket)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .inner
            .poll_close(cx)
            .map_err(Error::WebSocket)
    }
}

impl<T> Stream for WebSocketTransport<T>
where
    T: Stream<Item = Result<Message, tungstenite::Error>>,
{
    type Item = Result<ResponseEnvelope, Error>;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            match futures_util::ready!(this.inner.as_mut().poll_next(cx)) {
                Some(Ok(msg)) => match msg {
                    Message::Ping(..) => (),
                    Message::Text(s) => break Some(serde_json::from_str(&s).map_err(Error::Json)),
                    other => break Some(Err(Error::UnexpectedWebSocketMessage(other))),
                },
                Some(Err(e)) => break Some(Err(Error::WebSocket(e))),
                None => break None,
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

type MultiplexTransport<T> = tokio_tower::multiplex::MultiplexTransport<T, Tagger>;
type TransportError<T> = tokio_tower::Error<MultiplexTransport<T>, RequestEnvelope>;

pub struct Client<T>(MultiplexClient<MultiplexTransport<T>, TransportError<T>, RequestEnvelope>)
where
    T: Sink<RequestEnvelope> + TryStream;

pub async fn new(
    url: &str,
) -> Result<Client<WebSocketTransport<WebSocketStream<MaybeTlsStream<TcpStream>>>>> {
    let (ws, _) = tokio_tungstenite::connect_async(url).await?;

    Ok(Client::from_transport(WebSocketTransport::new(ws)))
}

impl<T> Client<T>
where
    T: Sink<RequestEnvelope, Error = Error>
        + TryStream<Ok = ResponseEnvelope, Error = Error>
        + Send
        + 'static,
{
    pub fn from_transport(underlying: T) -> Self {
        let client =
            MultiplexClient::with_error_handler(MultiplexTransport::new(underlying, Tagger), |e| {
                eprintln!("encountered error: {:?}", e)
            });
        Client(client)
    }

    pub async fn send<Req: Request>(&mut self, data: Req) -> Result<Req::Response> {
        let id = Uuid::new_v4().to_string();

        let msg = RequestEnvelope {
            api_name: "VTubeStudioPublicAPI".into(),
            api_version: "1.0".into(),
            request_id: Some(id),
            data: data.into(),
        };

        self.0
            .ready()
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        let resp = self
            .0
            .call(msg)
            .await
            .map_err(|e| Error::Transport(Box::new(e)))?;

        match Req::Response::try_from(resp.data) {
            Ok(data) => Ok(data),
            Err(ResponseData::ApiError(e)) => Err(Error::Api(e)),
            Err(e) => Err(Error::UnexpectedResponse {
                expected: Req::Response::MESSAGE_TYPE,
                received: e,
            }),
        }
    }
}
