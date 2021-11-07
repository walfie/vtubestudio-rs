use crate::data::ApiError;
use futures_core::TryStream;
use futures_sink::Sink;
use std::error::Error as StdError;
use std::fmt;

pub type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("service error: {0}")]
    Service(#[from] ServiceError),
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received})")]
    UnexpectedResponse {
        expected: &'static str,
        received: String,
    },
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct ServiceError {
    kind: ServiceErrorKind,
    source: Option<BoxError>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum ServiceErrorKind {
    #[error("no more in-flight requests allowed")]
    TransportFull,
    #[error("connection was dropped")]
    ConnectionDropped,
    #[error("received server response with unexpected request ID")]
    Desynchronized,
    #[error("underlying transport failed while attempting to receive a response")]
    Read,
    #[error("underlying transport failed to send a request")]
    Write,
    #[error("other error")]
    Other,
}

impl ServiceError {
    pub fn new(kind: ServiceErrorKind) -> Self {
        Self { kind, source: None }
    }

    /// Set this error's underlying `source`.
    pub fn with_source<E: Into<BoxError>>(mut self, source: E) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Consumes the error, returning its source.
    pub fn into_source(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.source
    }

    /// Convert a [`BoxError`] into this error type. If the underlying [`Error`](std::error::Error)
    /// is not this error type, a new [`Error`] is created with [`ServiceErrorKind::Other`].
    pub fn from_boxed(error: BoxError) -> Self {
        match error.downcast::<Self>() {
            Ok(e) => *e,
            Err(e) => Self::new(ServiceErrorKind::Other).with_source(e),
        }
    }

    pub fn kind(&self) -> &ServiceErrorKind {
        &self.kind
    }

    /// Check if any error in this error's `source` chain match the given [`ServiceErrorKind`].
    pub fn has_kind(&self, kind: ServiceErrorKind) -> bool {
        if self.kind == kind {
            return true;
        }

        let mut source = self.source();

        while let Some(e) = source {
            match e.downcast_ref::<Self>() {
                Some(ref found) if found.kind == kind => return true,
                _ => source = e.source(),
            }
        }

        false
    }

    /// Recurse through this error's `source` chain, returning the first matching error type.
    pub fn find_source<E: StdError + 'static>(&self) -> Option<&E> {
        let mut source = self.source();

        while let Some(e) = source {
            match e.downcast_ref::<E>() {
                Some(ref found) => return Some(found),
                None => source = e.source(),
            }
        }

        None
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref source) = self.source {
            write!(f, "{}: {}", self.kind, source)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl StdError for ServiceError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|cause| &**cause as &(dyn StdError + 'static))
    }
}

impl<T, I> From<tokio_tower::Error<T, I>> for ServiceError
where
    T: Sink<I> + TryStream,
    BoxError: From<<T as Sink<I>>::Error> + From<<T as TryStream>::Error>,
{
    fn from(error: tokio_tower::Error<T, I>) -> Self {
        use tokio_tower::Error::*;

        match error {
            BrokenTransportSend(e) => Self::new(ServiceErrorKind::Write).with_source(e),
            BrokenTransportRecv(Some(e)) => Self::new(ServiceErrorKind::Read).with_source(e),
            BrokenTransportRecv(None) | ClientDropped => {
                Self::new(ServiceErrorKind::ConnectionDropped)
            }
            TransportFull => Self::new(ServiceErrorKind::TransportFull),
            Desynchronized => Self::new(ServiceErrorKind::Desynchronized),
        }
    }
}
