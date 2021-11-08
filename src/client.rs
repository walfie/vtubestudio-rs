use crate::data::{Request, RequestEnvelope, ResponseEnvelope};
use crate::error::{Error, ServiceError};
use crate::service::TungsteniteApiService;

use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tower::{Service, ServiceExt};

#[derive(Clone, Debug)]
pub struct Client<S> {
    service: S,
}

pub type TungsteniteClient = Client<TungsteniteApiService>;
impl TungsteniteClient {
    pub async fn new_tungstenite<R>(request: R) -> Result<Self, tungstenite::Error>
    where
        R: IntoClientRequest + Send + Unpin,
    {
        Ok(Self::new(
            TungsteniteApiService::new_tungstenite(request).await?,
        ))
    }
}

pub async fn send_request<S, Req: Request>(
    service: &mut S,
    data: &Req,
) -> Result<Req::Response, Error>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    ServiceError: From<S::Error>,
{
    let msg = RequestEnvelope::new(data)?;

    let resp = service
        .ready()
        .await
        .map_err(ServiceError::from)?
        .call(msg)
        .await
        .map_err(ServiceError::from)?;

    resp.parse::<Req::Response>()
}

impl<S> Client<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    ServiceError: From<S::Error>,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }

    pub fn into_inner(self) -> S {
        self.service
    }

    pub async fn send<Req: Request>(&mut self, data: &Req) -> Result<Req::Response, Error> {
        send_request(&mut self.service, data).await
    }
}
