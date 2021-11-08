use crate::client::send_request;
use crate::data::{
    AuthenticationRequest, AuthenticationTokenRequest, RequestEnvelope, ResponseEnvelope,
};
use crate::error::{BoxError, Error, ServiceError, ServiceErrorKind};

use futures_util::TryFutureExt;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceExt};

#[derive(Clone)]
pub struct AuthenticationLayer {
    token: Option<String>,
    token_request: Arc<AuthenticationTokenRequest>,
}

impl AuthenticationLayer {
    pub fn new(token_request: AuthenticationTokenRequest) -> Self {
        Self {
            token_request: Arc::new(token_request),
            token: None,
        }
    }

    pub fn with_token<S: Into<Option<String>>>(mut self, token: S) -> Self {
        self.token = token.into();
        self
    }
}

impl<S> Layer<S> for AuthenticationLayer
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Send,
    ServiceError: From<S::Error>,
{
    type Service = Authentication<S>;

    fn layer(&self, service: S) -> Self::Service {
        Authentication::new(service, self.token_request.clone(), self.token.clone())
    }
}

#[derive(Clone)]
pub struct Authentication<S> {
    service: S,
    token: Arc<Mutex<Option<String>>>,
    token_request: Arc<AuthenticationTokenRequest>,
}

impl<S> Authentication<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Send,
    ServiceError: From<S::Error>,
{
    pub fn new(
        service: S,
        token_request: Arc<AuthenticationTokenRequest>,
        token: Option<String>,
    ) -> Self {
        Self {
            service,
            token_request,
            token: Arc::new(Mutex::new(token)),
        }
    }
}

impl<S> fmt::Debug for Authentication<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Avoid printing the token
        f.debug_struct("Authentication")
            .field("token", &"...")
            .field("token_request", &self.token_request)
            .field("service", &self.service)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ResponseWithToken {
    pub response: ResponseEnvelope,
    pub new_token: Option<String>,
}

impl ResponseWithToken {
    fn new(response: ResponseEnvelope) -> Self {
        ResponseWithToken {
            response,
            new_token: None,
        }
    }
}

pub async fn authenticate<S>(
    service: &mut S,
    stored_token: Option<String>,
    token_request: &AuthenticationTokenRequest,
) -> Result<Option<String>, Error>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    ServiceError: From<S::Error>,
{
    let (authentication_token, mut retry_on_fail) = match stored_token {
        Some(token) => (token, true),
        None => {
            let new_token = send_request(service, token_request)
                .await?
                .authentication_token;
            (new_token, false)
        }
    };

    let mut auth_req = AuthenticationRequest {
        plugin_name: token_request.plugin_name.clone(),
        plugin_developer: token_request.plugin_developer.clone(),
        authentication_token,
    };

    loop {
        let is_authenticated = send_request(service, &auth_req).await?.authenticated;

        if is_authenticated {
            return Ok(Some(auth_req.authentication_token.clone()));
        } else if retry_on_fail {
            let new_token = send_request(service, token_request)
                .await?
                .authentication_token;
            auth_req.authentication_token = new_token;
            retry_on_fail = false;
        } else {
            return Ok(None);
        }
    }
}

impl<S> Authentication<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    ServiceError: From<S::Error>,
{
    async fn authenticate(&mut self) -> Result<Option<String>, Error> {
        let stored_token = (*self.token.lock().unwrap()).clone();

        let new_token =
            authenticate(&mut self.service, stored_token, self.token_request.as_ref()).await?;

        if let Some(ref token) = new_token {
            *self.token.lock().unwrap() = Some(token.clone());
        }

        Ok(new_token)
    }
}

impl<S> Service<RequestEnvelope> for Authentication<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Send,
    ServiceError: From<S::Error>,
{
    type Response = ResponseWithToken;
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(ServiceError::from)
    }

    fn call(&mut self, req: RequestEnvelope) -> Self::Future {
        let mut this = self.clone();

        let f = async move {
            let resp = this
                .service
                .ready()
                .await
                .map_err(ServiceError::from)?
                .call(req)
                .map_err(ServiceError::from)
                .await?;

            if !resp.is_auth_error() {
                return Ok(ResponseWithToken::new(resp));
            }

            let new_token = this
                .authenticate()
                .map_err(|e| ServiceError::new(ServiceErrorKind::Authentication).with_source(e))
                .await?;

            Ok(ResponseWithToken {
                response: resp,
                new_token,
            })
        };

        Box::pin(f)
    }
}
