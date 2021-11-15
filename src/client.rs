use crate::data::{AuthenticationTokenRequest, Request, RequestEnvelope, ResponseEnvelope};
use crate::error::Error;
use crate::service::{
    send_request, AuthenticationLayer, CloneBoxService, ResponseWithToken, RetryPolicy,
};

use crate::error::BoxError;
use crate::service::CloneBoxApiService;
use std::borrow::Cow;
use std::error::Error as StdError;
use tokio::sync::mpsc;
use tower::reconnect::Reconnect;
use tower::{Service, ServiceBuilder};

/// A client for interacting with the VTube Studio API.
///
/// This is a wrapper on top of [`tower::Service`] that provides a convenient interface for
/// [`send`](Self::send)ing API requests and receiving structured data.
#[derive(Clone, Debug)]
pub struct Client<S = CloneBoxApiService> {
    service: S,
}

impl Client<CloneBoxApiService> {
    /// Creates a builder to configure a new client.
    ///
    /// # Example
    ///
    #[cfg_attr(feature = "tokio-tungstenite", doc = "```no_run")]
    #[cfg_attr(not(feature = "tokio-tungstenite"), doc = "```ignore")]
    /// # use vtubestudio::Client;
    /// let (mut client, mut new_tokens) = Client::builder()
    ///     .auth_token(Some("...".to_string()))
    ///     .authentication("Plugin name", "Developer name", None)
    ///     .build_tungstenite();
    /// ```
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

impl<S> Client<S>
where
    S: Service<RequestEnvelope, Response = ResponseEnvelope>,
    Error: From<S::Error>,
{
    /// Creates a new client from a [`Service`], if you want to provide your own custom middleware
    /// or transport. Most users will probably want to use the [`builder`](Client::builder) helper.
    pub fn new_from_service(service: S) -> Self {
        Self { service }
    }

    /// Consumes this client and returns the underlying [`Service`].
    pub fn into_service(self) -> S {
        self.service
    }

    /// Sends a VTube Studio API request.
    ///
    /// # Example
    ///
    #[cfg_attr(feature = "tokio-tungstenite", doc = "```no_run")]
    #[cfg_attr(not(feature = "tokio-tungstenite"), doc = "```ignore")]
    /// # async fn run() -> Result<(), vtubestudio::error::BoxError> {
    /// # use vtubestudio::Client;
    /// use vtubestudio::data::StatisticsRequest;
    ///
    /// # let (mut client, _) = Client::builder().build_tungstenite();
    /// let resp = client.send(&StatisticsRequest {}).await?;
    /// println!("VTube Studio has been running for {}ms", resp.uptime);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send<Req: Request>(&mut self, data: &Req) -> Result<Req::Response, Error> {
        send_request(&mut self.service, data).await
    }
}

/// A builder to configure a new [`Client`] with a set of recommended [`tower`] middleware.
///
/// * retrying requests on disconnect (using [`Reconnect`] and [`RetryPolicy`])
/// * if [`authentication`](Self::authentication) is provided, automatically reauthenticate and
///   retry when it encounters an auth error (using [`AuthenticationLayer`]).
///
/// # Example
///
#[cfg_attr(feature = "tokio-tungstenite", doc = "```no_run")]
#[cfg_attr(not(feature = "tokio-tungstenite"), doc = "```ignore")]
/// # async fn run() -> Result<(), vtubestudio::error::BoxError> {
/// # fn do_something_with_new_token(token: String) { unimplemented!(); }
/// # use vtubestudio::Client;
/// // An auth token from a previous successful authentication request
/// let stored_token = Some("...".to_string());
///
/// let (mut client, mut new_tokens) = Client::builder()
///     .authentication("Plugin name", "Developer name", None)
///     .auth_token(stored_token)
///     .build_tungstenite();
///
/// tokio::spawn(async move {
///     // This returns whenever the authentication middleware receives a new auth token.
///     // We can handle it by saving it somewhere, etc.
///     while let Some(token) = new_tokens.next().await {
///         do_something_with_new_token(token);
///     }
/// });
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    url: String,
    retry_on_disconnect: bool,
    request_buffer_size: usize,
    token_stream_buffer_size: usize,
    auth_token: Option<String>,
    token_request: Option<AuthenticationTokenRequest>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8001".to_string(),
            retry_on_disconnect: true,
            request_buffer_size: 256,
            token_stream_buffer_size: 32,
            auth_token: None,
            token_request: None,
        }
    }
}

/// A wrapper for a [`mpsc::Receiver`] that yields new auth tokens.
#[derive(Debug)]
pub struct TokenReceiver {
    receiver: mpsc::Receiver<String>,
}

impl TokenReceiver {
    /// Returns new tokens after successful authentication. If `None` is returned, it means the
    /// sender (the underlying [`Client`]) has been dropped.
    ///
    /// # Example
    ///
    #[cfg_attr(feature = "tokio-tungstenite", doc = "```no_run")]
    #[cfg_attr(not(feature = "tokio-tungstenite"), doc = "```ignore")]
    /// # async fn run() -> Result<(), vtubestudio::error::BoxError> {
    /// # fn do_something_with_new_token(token: String) { unimplemented!(); }
    /// # use vtubestudio::Client;
    /// let (mut client, mut new_tokens) = Client::builder()
    ///     .authentication("Plugin name", "Developer name", None)
    ///     .build_tungstenite();
    ///
    /// tokio::spawn(async move {
    ///     // This returns whenever the authentication middleware receives a new auth token.
    ///     // We can handle it by saving it somewhere, etc.
    ///     while let Some(token) = new_tokens.next().await {
    ///         do_something_with_new_token(token);
    ///     }
    /// });
    /// # Ok(())
    /// # }
    pub async fn next(&mut self) -> Option<String> {
        self.receiver.recv().await
    }

    /// Consume this receiver and return the underlying [`mpsc::Receiver`].
    pub fn into_inner(self) -> mpsc::Receiver<String> {
        self.receiver
    }
}

impl ClientBuilder {
    /// Creates new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    crate::cfg_feature! {
        #![feature = "tokio-tungstenite"]
        /// Consumes the builder and initializes a [`Client`] and [`TokenReceiver`] using
        /// [`tokio_tungstenite`] as the underlying websocket transport library.
        pub fn build_tungstenite(self) -> (Client, TokenReceiver) {
            use crate::service::MakeApiService;
            self.build_reconnecting_service(MakeApiService::new_tungstenite())
        }
    }

    /// If this is provided, whenever the underlying service encounters an authentication error, it
    /// will try to obtain a new auth token and retry the request.
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

    /// Sets the websocket URL. The default value is `ws://localhost:8001`.
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.url = url.into();
        self
    }

    /// Initial token to use for reauthentication (if [`authentication`](Self::authentication) is
    /// provided). This should be the result of a previous successful authentication attempt.
    pub fn auth_token(mut self, token: Option<String>) -> Self {
        self.auth_token = token;
        self
    }

    /// Retry requests on disconnect. The default value is `true`.
    pub fn retry_on_disconnect(mut self, retry: bool) -> Self {
        self.retry_on_disconnect = retry;
        self
    }

    /// The max number of outstanding requests. The default value is `256`.
    pub fn request_buffer_size(mut self, size: usize) -> Self {
        self.request_buffer_size = size;
        self
    }

    /// The max capacity of the [`TokenReceiver`] buffer. This represents the max number of
    /// unacknowledged new tokens before client stops sending tokens. The default value is `32`.
    pub fn token_stream_buffer_size(mut self, size: usize) -> Self {
        self.token_stream_buffer_size = size;
        self
    }

    /// Consumes the builder and initializes a [`Client`] and [`TokenReceiver`] using a custom
    /// [`Service`].
    pub fn build_service<S>(self, service: S) -> (Client, TokenReceiver)
    where
        S: Service<RequestEnvelope, Response = ResponseEnvelope> + Send + 'static,
        S::Error: Into<BoxError> + Send + Sync,
        S::Future: Send,
    {
        let policy = RetryPolicy::new()
            .on_disconnect(self.retry_on_disconnect)
            .on_auth_error(self.token_request.is_some());

        let (token_tx, token_rx) = mpsc::channel(self.token_stream_buffer_size);

        let service = if let Some(token_req) = self.token_request {
            CloneBoxService::new(
                ServiceBuilder::new()
                    .retry(policy)
                    .and_then(|resp: ResponseWithToken| async move {
                        if let Some(token) = resp.new_token {
                            // Ignore send errors (the consumer probably isn't reading the new
                            // token stream)
                            let _ = token_tx.send(token).await;
                        }
                        Ok(resp.response)
                    })
                    .layer(AuthenticationLayer::new(token_req).with_token(self.auth_token))
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

        let token_receiver = TokenReceiver { receiver: token_rx };

        return (Client::new_from_service(service), token_receiver);
    }

    /// Consumes the builder and initializes a [`Client`] and [`TokenReceiver`] with a reconnecting
    /// service.
    ///
    /// The input service should be a [`MakeService`](tower::MakeService) that satisfies the
    /// requirements of [`Reconnect`].
    pub fn build_reconnecting_service<S>(self, service: S) -> (Client, TokenReceiver)
    where
        S: Service<String> + Send + 'static,
        S::Error: StdError + Send + Sync,
        S::Future: Send + Unpin,
        S::Response: Service<RequestEnvelope, Response = ResponseEnvelope> + Send + 'static,
        <S::Response as Service<RequestEnvelope>>::Error: StdError + Send + Sync,
        <S::Response as Service<RequestEnvelope>>::Future: Send,
    {
        let service = Reconnect::new::<S, String>(service, self.url.clone());

        self.build_service(service)
    }
}
