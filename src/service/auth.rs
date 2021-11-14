use crate::data::{
    AuthenticationRequest, AuthenticationTokenRequest, RequestEnvelope, ResponseEnvelope,
};
use crate::error::Error;
use crate::service::send_request;

use futures_util::TryFutureExt;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tower::{Layer, Service, ServiceExt};

/// A [`Layer`] that produces an [`Authentication`] service.
#[derive(Clone)]
pub struct AuthenticationLayer {
    token: Option<String>,
    token_request: Arc<AuthenticationTokenRequest>,
}

impl fmt::Debug for AuthenticationLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Avoid printing the token
        f.debug_struct("AuthenticationLayer")
            .field("token", &self.token.is_some().then(|| "..."))
            .field("token_request", &self.token_request)
            .finish()
    }
}

impl AuthenticationLayer {
    /// Creates a new [`AuthenticationLayer`] with the given developer info.
    pub fn new(token_request: AuthenticationTokenRequest) -> Self {
        Self {
            token_request: Arc::new(token_request),
            token: None,
        }
    }

    /// Provides the [`Authentication`] service with an existing auth token.
    ///
    /// On auth errors, the [`Authentication`] service will attempt to use this token first before
    /// trying to request a new one.
    pub fn with_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }
}

impl<S> Layer<S> for AuthenticationLayer
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Send,
    Error: From<S::Error>,
{
    type Service = Authentication<S>;

    fn layer(&self, service: S) -> Self::Service {
        Authentication::new(service, self.token_request.clone(), self.token.clone())
    }
}

/// A [`Service`] that tries to reauthenticate when receiving an auth error.
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
    Error: From<S::Error>,
{
    /// Creates a new [`Authentication`] service.
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

/// Wrapper struct containing the API response and an optional token, if a new token was obtained
/// via a successful authentication token request.
#[derive(Debug, Clone)]
pub struct ResponseWithToken {
    /// The underlying API response.
    pub response: ResponseEnvelope,
    /// New auth token received by the [`Authentication`] middleware, if any.
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

/// Attempt to authenticate using the provided credentials.
///
/// Returns `Ok(_)` if the request succeeded, and `Ok(Some(new_token))` if the request succeeded
/// and a new token was obtained.
pub async fn authenticate<S>(
    service: &mut S,
    stored_token: Option<String>,
    token_request: &AuthenticationTokenRequest,
) -> Result<Option<String>, Error>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    Error: From<S::Error>,
{
    let mut is_new_token = false;
    let (authentication_token, mut retry_on_fail) = match stored_token {
        Some(token) => (token, true),
        None => {
            let new_token = send_request(service, token_request)
                .await?
                .authentication_token;
            is_new_token = true;
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
            return Ok(is_new_token.then(|| auth_req.authentication_token.clone()));
        } else if retry_on_fail {
            let new_token = send_request(service, token_request)
                .await?
                .authentication_token;
            is_new_token = true;
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
    Error: From<S::Error>,
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
    Error: From<S::Error>,
{
    type Response = ResponseWithToken;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(Error::from)
    }

    fn call(&mut self, req: RequestEnvelope) -> Self::Future {
        let mut this = self.clone();

        let f = async move {
            let resp = this
                .service
                .ready()
                .await
                .map_err(Error::from)?
                .call(req)
                .map_err(Error::from)
                .await?;

            if !resp.is_unauthenticated_error() {
                return Ok(ResponseWithToken::new(resp));
            }

            let new_token = this.authenticate().await?;

            Ok(ResponseWithToken {
                response: resp,
                new_token,
            })
        };

        Box::pin(f)
    }
}
