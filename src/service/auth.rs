use crate::data::{
    AuthenticationRequest, AuthenticationTokenRequest, RequestEnvelope, ResponseEnvelope,
};
use crate::error::{Error, ErrorKind};
use crate::service::send_request;

use futures_util::TryFutureExt;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
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

/// A [`Service`] that handles the VTube Studio authentication flow internally.
///
/// This service will try to authenticate using a stored token after:
///
/// * the initial connection is established
/// * encountering a disconnection error
/// * receiving an auth error from the API
///
/// If no stored token is availble, or the token is invalid, it will request a new auth token by
/// sending an [`AuthenticationTokenRequest`] (which will require the user to accept the pop-up in
/// the VTube Studio app).
#[derive(Clone)]
pub struct Authentication<S> {
    service: S,
    token: Arc<Mutex<Option<String>>>,
    token_request: Arc<AuthenticationTokenRequest>,
    is_authenticated: Arc<AtomicBool>,
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
            is_authenticated: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<S> Authentication<S> {
    /// Consumes `self`, returning the inner service.
    pub fn into_inner(self) -> S {
        self.service
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
            .field("is_authenticated", &self.is_authenticated)
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
    // Helper for authenticating using a stored token, and managing internal state (updating
    // current authentication status and storing new tokens).
    async fn authenticate(&mut self) -> Result<Option<String>, Error> {
        let stored_token = (*self.token.lock().unwrap()).clone();

        let token_result =
            authenticate(&mut self.service, stored_token, self.token_request.as_ref()).await;

        let new_token = match token_result {
            Ok(maybe_token) => {
                if maybe_token.is_some() {
                    *self.token.lock().unwrap() = maybe_token.clone();
                }

                self.set_authentication_status(true);
                maybe_token
            }
            Err(e) => {
                self.set_authentication_status(false);
                return Err(e);
            }
        };

        Ok(new_token)
    }

    fn set_authentication_status(&mut self, is_authenticated: bool) {
        self.is_authenticated
            .store(is_authenticated, Ordering::Relaxed);
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
            // Attempt to authenticate if we aren't already (on initial connection, after
            // disconnects, after unrecoverable auth failures, etc)
            let mut new_token = if !this.is_authenticated.load(Ordering::Relaxed) {
                this.authenticate().await?
            } else {
                None
            };

            let response = match this.service.ready().and_then(|s| s.call(req)).await {
                Ok(resp) => resp,
                Err(e) => {
                    let error = Error::from(e);
                    if error.has_kind(ErrorKind::ConnectionDropped) {
                        this.set_authentication_status(false);
                    }
                    return Err(error);
                }
            };

            if response.is_unauthenticated_error() {
                new_token = this.authenticate().await?;
            }

            Ok(ResponseWithToken {
                response,
                new_token,
            })
        };

        Box::pin(f)
    }
}
