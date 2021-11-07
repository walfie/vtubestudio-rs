use crate::data::{
    AuthenticationRequest, AuthenticationTokenRequest, Request, RequestEnvelope, ResponseEnvelope,
};
use crate::error::{Error, ServiceError};
use crate::service::TungsteniteApiService;

use std::fmt;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tower::{Service, ServiceExt};

#[derive(Clone)]
pub struct Client<S> {
    service: S,
    token: Option<String>,
    auth_request: Option<AuthenticationTokenRequest>,
}

impl<S> fmt::Debug for Client<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Avoid printing the token
        let token = self.token.is_some().then(|| "...");

        f.debug_struct("Client")
            .field("token", &token)
            .field("auth_request", &self.auth_request)
            .field("service", &self.service)
            .finish()
    }
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
        Self {
            service,
            token: None,
            auth_request: None,
        }
    }

    pub fn with_token(&mut self, token: String) -> &mut Self {
        self.token = Some(token);
        self
    }

    pub fn with_auth_request(&mut self, auth_request: AuthenticationTokenRequest) -> &mut Self {
        self.auth_request = Some(auth_request);
        self
    }

    pub fn into_inner(self) -> S {
        self.service
    }

    pub async fn send<Req: Request>(&mut self, data: &Req) -> Result<Req::Response, Error> {
        let auth_error = match send_request(&mut self.service, data).await {
            Ok(resp) => return Ok(resp),
            Err(e) if !e.is_auth_error() => return Err(e),
            Err(other) => other,
        };

        let auth_token_req = match &self.auth_request {
            Some(req) => req,
            None => return Err(auth_error),
        };

        let mut auth_req = AuthenticationRequest {
            plugin_name: auth_token_req.plugin_name.clone(),
            plugin_developer: auth_token_req.plugin_developer.clone(),
            authentication_token: "".into(),
        };

        if let Some(ref token) = self.token {
            auth_req.authentication_token = token.clone();
            if let Ok(_) = send_request(&mut self.service, &auth_req).await {
                return send_request(&mut self.service, data).await;
            } else {
                return Err(auth_error);
            }
        }

        if let Ok(resp) = send_request(&mut self.service, auth_token_req).await {
            let new_token = resp.authentication_token;
            auth_req.authentication_token = new_token.clone();
            self.token = Some(new_token);
        }

        if let Ok(_) = send_request(&mut self.service, &auth_req).await {
            return send_request(&mut self.service, data).await;
        } else {
            return Err(auth_error);
        }
    }
}
