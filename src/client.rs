use crate::data::{AuthenticationTokenRequest, Request, RequestEnvelope, ResponseEnvelope};
use crate::error::Error;
use crate::service::{
    AuthenticationLayer, MakeApiService, ResponseWithToken, RetryPolicy, TungsteniteApiService,
};
use crate::CloneBoxService;

use std::borrow::Cow;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tower::reconnect::Reconnect;
use tower::{Service, ServiceBuilder, ServiceExt};

#[derive(Clone, Debug)]
pub struct Client<S = CloneBoxApiService> {
    service: S,
}

pub type CloneBoxApiService = CloneBoxService<RequestEnvelope, ResponseEnvelope, Error>;

/// Trait alias for a [`tower::Service`] that is compatible with [`Client`].
pub trait ClientService:
    Service<RequestEnvelope, Response = ResponseEnvelope> + Send + Sync
where
    Error: From<Self::Error>,
{
}

impl<S> ClientService for S
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope> + Send + Sync,
    Error: From<Self::Error>,
{
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
    Error: From<S::Error>,
{
    let msg = RequestEnvelope::new(data)?;

    let resp = service
        .ready()
        .await
        .map_err(Error::from)?
        .call(msg)
        .await
        .map_err(Error::from)?;

    resp.parse::<Req::Response>()
}

impl<S> Client<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    Error: From<S::Error>,
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

impl Client<CloneBoxApiService> {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

#[derive(Debug, Clone)]
pub struct ClientBuilder {
    retry_on_disconnect: bool,
    retry_on_reauthentication: bool,
    request_buffer_size: usize,
    auth_token: Option<String>,
    token_request: Option<AuthenticationTokenRequest>,
    // TODO: function to be called on new token
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            retry_on_disconnect: true,
            retry_on_reauthentication: true,
            request_buffer_size: 256,
            auth_token: None,
            token_request: None,
        }
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retry_on_reauthentication(mut self, retry: bool) -> Self {
        self.retry_on_reauthentication = retry;
        self
    }

    pub fn retry_on_disconnect(mut self, retry: bool) -> Self {
        self.retry_on_disconnect = retry;
        self
    }

    pub fn request_buffer_size(mut self, size: usize) -> Self {
        self.request_buffer_size = size;
        self
    }

    pub fn auth_token<T: Into<Option<String>>>(mut self, token: T) -> Self {
        self.auth_token = token.into();
        self
    }

    pub fn authentication<S1, S2, S3>(mut self, name: S1, developer: S2, icon: S3) -> Self
    where
        S1: Into<Cow<'static, str>>,
        S2: Into<Cow<'static, str>>,
        S3: Into<Option<Cow<'static, str>>>,
    {
        self.token_request = Some(AuthenticationTokenRequest {
            plugin_name: name.into(),
            plugin_developer: developer.into(),
            plugin_icon: icon.into(),
        });
        self
    }

    pub fn build_tungstenite<R>(
        self,
        request: R,
    ) -> Client<CloneBoxService<RequestEnvelope, ResponseEnvelope, Error>>
    where
        R: IntoClientRequest + Clone + Send + Unpin + 'static,
    {
        let policy = RetryPolicy::new()
            .on_disconnect(self.retry_on_disconnect)
            .on_auth_error(self.retry_on_reauthentication);

        let service =
            Reconnect::new::<TungsteniteApiService, R>(MakeApiService::new_tungstenite(), request);

        let service = if let Some(token_req) = self.token_request {
            CloneBoxService::new(
                ServiceBuilder::new()
                    .retry(policy)
                    .map_response(|resp: ResponseWithToken| resp.response)
                    .layer(AuthenticationLayer::new(token_req))
                    .map_err(Error::from_boxed)
                    .buffer(self.request_buffer_size)
                    .service(service),
            )
        } else {
            CloneBoxService::new(
                ServiceBuilder::new()
                    .retry(policy)
                    .map_err(Error::from_boxed)
                    .buffer(self.request_buffer_size)
                    .service(service),
            )
        };

        return Client::new(service);
    }
}
