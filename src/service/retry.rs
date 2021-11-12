use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::{Error, ErrorKind};

use futures_util::future;
use tower::retry::{Policy, Retry};
use tower::Layer;

/// Determines whether a request should be retried.
///
/// This can be used as either a [`Layer`] or a [`Policy`].
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    retry_on_disconnect: bool,
    retry_on_auth_error: bool,
}

impl RetryPolicy {
    /// Creates a new [`RetryPolicy`] with default values.
    pub fn new() -> Self {
        RetryPolicy {
            retry_on_disconnect: false,
            retry_on_auth_error: false,
        }
    }

    /// Whether requests should be retried on disconnect. Default `false`.
    pub fn on_disconnect(mut self, value: bool) -> Self {
        self.retry_on_disconnect = value;
        self
    }

    /// Whether requests should be retried on auth error. Default `false`.
    pub fn on_auth_error(mut self, value: bool) -> Self {
        self.retry_on_auth_error = value;
        self
    }
}

impl<S> Layer<S> for RetryPolicy {
    type Service = Retry<Self, S>;

    fn layer(&self, service: S) -> Self::Service {
        let policy = self.clone();
        Retry::new(policy, service)
    }
}

impl Policy<RequestEnvelope, ResponseEnvelope, Error> for RetryPolicy {
    type Future = future::Ready<Self>;

    fn retry(
        &self,
        _req: &RequestEnvelope,
        result: Result<&ResponseEnvelope, &Error>,
    ) -> Option<Self::Future> {
        Some(future::ready(match result {
            Ok(resp) if resp.is_auth_error() && self.retry_on_auth_error => {
                self.clone().on_auth_error(false)
            }

            Err(e) if self.retry_on_disconnect && e.has_kind(ErrorKind::ConnectionDropped) => {
                self.clone().on_disconnect(false)
            }
            _ => return None,
        }))
    }

    fn clone_request(&self, req: &RequestEnvelope) -> Option<RequestEnvelope> {
        Some(req.clone())
    }
}
