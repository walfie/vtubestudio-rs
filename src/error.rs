use futures_core::TryStream;
use futures_sink::Sink;
use std::error::Error as StdError;
use std::fmt;

pub use crate::data::ApiError;
pub type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T, E = ServiceError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct ServiceError {
    kind: ServiceErrorKind,
    source: Option<BoxError>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum ServiceErrorKind {
    #[error("received APIError from server")]
    Api,
    #[error("no more in-flight requests allowed")]
    TransportFull,
    #[error("failed to establish connection")]
    ConnectionRefused,
    #[error("connection was dropped")]
    ConnectionDropped,
    #[error("received unexpected response from server")]
    UnexpectedResponse,
    #[error("received server response with unexpected request ID")]
    Desynchronized,
    #[error("JSON error")]
    Json,
    #[error("underlying transport failed while attempting to receive a response")]
    Read,
    #[error("underlying transport failed to send a request")]
    Write,
    #[error("authentication failed")]
    Authentication,
    #[error("other error")]
    Other,
}

#[derive(thiserror::Error, Debug)]
#[error("received unexpected response (expected {expected}, received {received})")]
pub struct UnexpectedResponseError {
    pub expected: &'static str,
    pub received: String,
}

impl From<serde_json::Error> for ServiceError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(ServiceErrorKind::Json).with_source(error)
    }
}

impl From<ApiError> for ServiceError {
    fn from(error: ApiError) -> Self {
        Self::new(ServiceErrorKind::Api).with_source(error)
    }
}

impl From<UnexpectedResponseError> for ServiceError {
    fn from(error: UnexpectedResponseError) -> Self {
        Self::new(ServiceErrorKind::UnexpectedResponse).with_source(error)
    }
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

    pub fn as_api_error(&self) -> Option<&ApiError> {
        self.find_source::<ApiError>()
    }

    pub fn is_api_error(&self) -> bool {
        self.as_api_error().is_some()
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(self.as_api_error(), Some(e) if e.is_auth_error())
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
