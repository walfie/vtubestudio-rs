use crate::data::{Request, RequestEnvelope, Response, ResponseData, ResponseEnvelope};
use crate::error::{ClientError, Error};
use crate::service::{ApiService, TungsteniteApiService};
use crate::transport::ApiTransport;

use std::convert::TryFrom;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tower::{Service, ServiceExt};

#[derive(Debug, Clone)]
pub struct Client<S> {
    inner: S,
}

pub type TungsteniteClient = Client<TungsteniteApiService>;
impl TungsteniteClient {
    pub async fn new_tungstenite<R>(request: R) -> Result<Self, tungstenite::Error>
    where
        R: IntoClientRequest + Send + Unpin,
    {
        let (ws, _) = tokio_tungstenite::connect_async(request).await?;
        let transport = ApiTransport::new_tungstenite(ws);
        let service = ApiService::new(transport);
        Ok(Self::new(service))
    }
}

impl<S> Client<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    Error: From<S::Error>,
{
    pub fn new(service: S) -> Self {
        Self { inner: service }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub async fn send<Req: Request>(&mut self, data: Req) -> Result<Req::Response, ClientError> {
        let msg = RequestEnvelope::new(data.into());

        let resp = self
            .inner
            .ready()
            .await
            .map_err(Error::from)?
            .call(msg)
            .await
            .map_err(Error::from)?;

        match Req::Response::try_from(resp.data) {
            Ok(data) => Ok(data),
            Err(ResponseData::ApiError(e)) => Err(ClientError::Api(e)),
            Err(e) => Err(ClientError::UnexpectedResponse {
                expected: Req::Response::MESSAGE_TYPE,
                received: e,
            }),
        }
    }
}
