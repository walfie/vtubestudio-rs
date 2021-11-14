use futures_core::TryStream;
use futures_sink::Sink;
use std::error::Error as StdError;

pub use crate::data::{ApiError, ArbitraryResponseType};

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn StdError + Send + Sync>;

/// Result type often returned from methods that can have [`vtubestudio::Error`](Error)s.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Represents errors that can occur while communicating with the VTube Studio API.
#[derive(thiserror::Error, Debug)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
    source: Option<BoxError>,
}

/// Describes the type of underlying error.
#[derive(thiserror::Error, displaydoc::Display, Debug, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// received APIError from server
    Api,
    /// no more in-flight requests allowed
    TransportFull,
    /// failed to establish connection
    ConnectionRefused,
    /// connection was dropped
    ConnectionDropped,
    /// received unexpected response from server
    UnexpectedResponse,
    /// received server response with unexpected request ID
    Desynchronized,
    /// JSON error
    Json,
    /// underlying transport failed while attempting to receive a response
    Read,
    /// underlying transport failed to send a request
    Write,
    /// other error
    Other,
}

/// The API response type did not match the expected type.
#[derive(thiserror::Error, Debug)]
#[error("received unexpected response (expected {expected}, received {received})")]
pub struct UnexpectedResponseError {
    /// The expected response type.
    pub expected: ArbitraryResponseType,
    /// The received response type.
    pub received: ArbitraryResponseType,
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
    /// Creates a new [`Error`].
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    /// Returns the underlying [`ApiError`], if any.
    pub fn to_api_error(&self) -> Option<&ApiError> {
        self.find_source::<ApiError>()
    }

    /// Sets this error's underlying `source`.
    pub fn with_source<E: Into<BoxError>>(mut self, source: E) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Consumes the error, returning its source.
    pub fn into_source(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.source
    }

    /// Returns `true` if this error has an underlying [`ApiError`].
    pub fn is_api_error(&self) -> bool {
        self.to_api_error().is_some()
    }

    /// Returns `true` if this error's underlying [`ApiError`] is an authentication error.
    pub fn is_auth_error(&self) -> bool {
        matches!(self.to_api_error(), Some(e) if e.is_auth_error())
    }

    /// Converts a [`BoxError`] into this error type. If the underlying [`Error`](std::error::Error)
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

    /// Checks if any error in this error's `source` chain matches the given [`ErrorKind`].
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

    /// Recurses through this error's `source` chain, returning the first matching error type.
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

#[doc(hidden)]
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
