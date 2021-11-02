pub(crate) mod id;

pub use crate::client::id::{IdTagger, NumericIdGenerator, RequestIdGenerator};

use crate::data::{Request, RequestEnvelope, Response, ResponseData};
use crate::error::Error;
use crate::transport::{ApiTransport, WebSocketTransport};

use std::convert::TryFrom;
use tokio_tower::multiplex::{Client as MultiplexClient, MultiplexTransport};
use tower::util::ServiceExt;
use tower::Service;

#[derive(Debug)]
pub struct Client<T: WebSocketTransport> {
    client: MultiplexClient<
        MultiplexTransport<ApiTransport<T>, IdTagger>,
        crate::Error<T>,
        RequestEnvelope,
    >,
}

impl<T> Client<T>
where
    T: WebSocketTransport,
{
    pub fn new(ws_transport: T::Underlying) -> Self {
        let tagger = IdTagger {
            id_generator: NumericIdGenerator::new(),
        };

        let api_transport = ApiTransport::new(ws_transport);

        let multiplex_transport = MultiplexTransport::new(api_transport, tagger);

        let client = MultiplexClient::new(multiplex_transport);
        Self { client }
    }

    pub async fn send<Req: Request>(&mut self, data: Req) -> Result<Req::Response, Error<T>> {
        let msg = RequestEnvelope {
            api_name: "VTubeStudioPublicAPI".into(),
            api_version: "1.0".into(),
            request_id: None,
            data: data.into(),
        };

        let resp = self.client.ready().await?.call(msg).await?;

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
