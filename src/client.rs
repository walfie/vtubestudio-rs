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
    token_request: Option<AuthenticationTokenRequest>,
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
            .field("token_request", &self.token_request)
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
            token_request: None,
        }
    }

    pub fn with_token<T>(mut self, token: String) -> Self
    where
        T: Into<Option<String>>,
    {
        self.token = token.into();
        self
    }

    pub fn with_token_request<A>(mut self, token_request: A) -> Self
    where
        A: Into<Option<AuthenticationTokenRequest>>,
    {
        self.token_request = token_request.into();
        self
    }

    pub fn into_inner(self) -> S {
        self.service
    }

    pub async fn send<Req: Request>(&mut self, data: &Req) -> Result<Req::Response, Error> {
        send_request(&mut self.service, data).await

        /*
        let original_err = match send_request(&mut self.service, data).await {
            Ok(resp) => return Ok(resp),
            Err(e) if !e.is_auth_error() => return Err(e),
            Err(other) => other,
        };

        let token_req = match &self.token_request {
            Some(req) => req,
            None => return Err(original_err),
        };

        let (authentication_token, mut retry_on_fail) = match self.token {
            Some(ref stored_token) => (stored_token.clone(), true),
            None => {
                let new_token = send_request(&mut self.service, token_req)
                    .await?
                    .authentication_token;
                self.token = Some(new_token.clone());
                (new_token, false)
            }
        };

        let mut auth_req = AuthenticationRequest {
            plugin_name: token_req.plugin_name.clone(),
            plugin_developer: token_req.plugin_developer.clone(),
            authentication_token,
        };

        loop {
            let is_authenticated = send_request(&mut self.service, &auth_req)
                .await?
                .authenticated;

            if is_authenticated {
                // Resend the original request
                return send_request(&mut self.service, data).await;
            }

            if retry_on_fail {
                let new_token = send_request(&mut self.service, token_req)
                    .await?
                    .authentication_token;
                self.token = Some(new_token.clone());
                auth_req.authentication_token = new_token;
                retry_on_fail = false;
            } else {
                return Err(original_err);
            }
        }
            */
    }
}
