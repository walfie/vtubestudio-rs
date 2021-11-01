mod transport;
pub use crate::client::transport::WebSocketTransport;
pub use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::data::*;
use crate::error::{Error, Result};

use futures_core::stream::TryStream;
use futures_core::Stream;
use futures_sink::Sink;
use std::convert::TryFrom;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_tower::multiplex::{Client as MultiplexClient, TagStore};
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

type MultiplexTransport<T> = tokio_tower::multiplex::MultiplexTransport<T, Tagger>;
type TransportError<T> = tokio_tower::Error<MultiplexTransport<T>, RequestEnvelope>;

pub struct Client<T>(MultiplexClient<MultiplexTransport<T>, TransportError<T>, RequestEnvelope>)
where
    T: Sink<RequestEnvelope> + TryStream;

pub type TungsteniteClient = Client<WebSocketTransport<WebSocketStream<MaybeTlsStream<TcpStream>>>>;
impl TungsteniteClient {
    pub async fn connect<R>(url: R) -> Result<Self>
    where
        R: IntoClientRequest + Unpin,
    {
        let (ws, _) = tokio_tungstenite::connect_async(url).await?;

        Ok(Client::from_transport(WebSocketTransport::new(ws)))
    }
}

impl<T> Client<T>
where
    T: Sink<RequestEnvelope, Error = Error>
        + Stream<Item = Result<ResponseEnvelope, Error>>
        + Send
        + 'static,
{
    pub fn from_transport(underlying: T) -> Self {
        let client =
            MultiplexClient::with_error_handler(MultiplexTransport::new(underlying, Tagger), |e| {
                // TODO
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
