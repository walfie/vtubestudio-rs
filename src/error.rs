use futures_core::TryStream;
use futures_sink::Sink;
use std::error::Error as StdError;

pub use crate::data::ApiError;
pub type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
#[error("{}", .kind)]
pub struct Error {
    kind: ErrorKind,
    source: Option<BoxError>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
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
    #[error("other error")]
    Other,
}

#[derive(thiserror::Error, Debug)]
#[error("received unexpected response (expected {expected}, received {received})")]
pub struct UnexpectedResponseError {
    pub expected: &'static str,
    pub received: String,
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::new(ErrorKind::Json).with_source(error)
    }
}

impl From<ApiError> for Error {
    fn from(error: ApiError) -> Self {
        Self::new(ErrorKind::Api).with_source(error)
    }
}

impl From<UnexpectedResponseError> for Error {
    fn from(error: UnexpectedResponseError) -> Self {
        Self::new(ErrorKind::UnexpectedResponse).with_source(error)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
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

    /// Return the underlying [`ApiError`], if any.
    pub fn as_api_error(&self) -> Option<&ApiError> {
        self.find_source::<ApiError>()
    }

    /// Returns `true` if this error has an underlying [`ApiError`].
    pub fn is_api_error(&self) -> bool {
        self.as_api_error().is_some()
    }

    /// Returns `true` if this error's underlying [`ApiError`] is an authentication error.
    pub fn is_auth_error(&self) -> bool {
        matches!(self.as_api_error(), Some(e) if e.is_auth_error())
    }

    /// Convert a [`BoxError`] into this error type. If the underlying [`Error`](std::error::Error)
    /// is not this error type, a new [`Error`] is created with [`ErrorKind::Other`].
    pub fn from_boxed(error: BoxError) -> Self {
        match error.downcast::<Self>() {
            Ok(e) => *e,
            Err(e) => Self::new(ErrorKind::Other).with_source(e),
        }
    }

    /// Returns the [`ErrorKind`] of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Check if any error in this error's `source` chain match the given [`ErrorKind`].
    pub fn has_kind(&self, kind: ErrorKind) -> bool {
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

impl<T, I> From<tokio_tower::Error<T, I>> for Error
where
    T: Sink<I> + TryStream,
    BoxError: From<<T as Sink<I>>::Error> + From<<T as TryStream>::Error>,
{
    fn from(error: tokio_tower::Error<T, I>) -> Self {
        use tokio_tower::Error::*;

        match error {
            BrokenTransportSend(e) => Self::new(ErrorKind::Write).with_source(e),
            BrokenTransportRecv(Some(e)) => Self::new(ErrorKind::Read).with_source(e),
            BrokenTransportRecv(None) | ClientDropped => Self::new(ErrorKind::ConnectionDropped),
            TransportFull => Self::new(ErrorKind::TransportFull),
            Desynchronized => Self::new(ErrorKind::Desynchronized),
        }
    }
}
