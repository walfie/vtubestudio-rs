use crate::data::{RequestEnvelope, ResponseEnvelope};
use crate::error::{ServiceError, ServiceErrorKind};

use futures_util::future;
use tower::retry::{Policy, Retry};
use tower::Layer;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    attempts_left: usize,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize) -> Self {
        RetryPolicy {
            attempts_left: max_attempts,
        }
    }

    pub fn once() -> Self {
        Self::new(1)
    }
}

impl Policy<RequestEnvelope, ResponseEnvelope, ServiceError> for RetryPolicy {
    type Future = future::Ready<Self>;

    fn retry(
        &self,
        _req: &RequestEnvelope,
        result: Result<&ResponseEnvelope, &ServiceError>,
    ) -> Option<Self::Future> {
        let e = result.err()?;

        if self.attempts_left > 0 && e.has_kind(ServiceErrorKind::ConnectionDropped) {
            Some(future::ready(Self {
                attempts_left: self.attempts_left - 1,
            }))
        } else {
            None
        }
    }

    fn clone_request(&self, req: &RequestEnvelope) -> Option<RequestEnvelope> {
        Some(req.clone())
    }
}

#[derive(Debug, Clone)]
pub struct RetryLayer {
    policy: RetryPolicy,
}

impl RetryLayer {
    pub fn new(max_attempts: usize) -> Self {
        RetryLayer {
            policy: RetryPolicy::new(max_attempts),
        }
    }

    pub fn once() -> Self {
        Self::new(1)
    }
}

impl<S> Layer<S> for RetryLayer {
    type Service = Retry<RetryPolicy, S>;

    fn layer(&self, service: S) -> Self::Service {
        let policy = self.policy.clone();
        Retry::new(policy, service)
    }
}
