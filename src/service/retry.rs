use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::{ServiceError, ServiceErrorKind};

use futures_util::future;
use tower::retry::{Policy, Retry};
use tower::Layer;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    retry_on_disconnect: bool,
    retry_on_auth_error: bool,
}

impl RetryPolicy {
    pub fn new() -> Self {
        RetryPolicy {
            retry_on_disconnect: false,
            retry_on_auth_error: false,
        }
    }

    pub fn on_disconnect(mut self, value: bool) -> Self {
        self.retry_on_disconnect = value;
        self
    }

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

impl Policy<RequestEnvelope, ResponseEnvelope, ServiceError> for RetryPolicy {
    type Future = future::Ready<Self>;

    fn retry(
        &self,
        _req: &RequestEnvelope,
        result: Result<&ResponseEnvelope, &ServiceError>,
    ) -> Option<Self::Future> {
        Some(future::ready(match result {
            Ok(resp) if resp.is_auth_error() && self.retry_on_auth_error => {
                self.clone().on_auth_error(false)
            }

            Err(e)
                if self.retry_on_disconnect && e.has_kind(ServiceErrorKind::ConnectionDropped) =>
            {
                self.clone().on_disconnect(false)
            }
            _ => return None,
        }))
    }

    fn clone_request(&self, req: &RequestEnvelope) -> Option<RequestEnvelope> {
        Some(req.clone())
    }
}
