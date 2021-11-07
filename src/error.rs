use crate::data::{ApiError, ResponseData};
use futures_core::TryStream;
use futures_sink::Sink;
use std::error::Error as StdError;
use std::fmt;

pub type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("service error: {0}")]
    Service(#[from] Error),
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received:?})")]
    UnexpectedResponse {
        expected: &'static str,
        received: ResponseData,
    },
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<BoxError>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
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

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error { kind, source: None }
    }

    pub fn with_source<E: Into<BoxError>>(mut self, source: E) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Consumes the error, returning its source.
    pub fn into_source(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.source
    }

    pub fn from_boxed(error: BoxError) -> Self {
        match error.downcast::<Error>() {
            Ok(e) => *e,
            Err(e) => Self::new(ErrorKind::Other).with_source(e),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn has_kind(&self, kind: ErrorKind) -> bool {
        if self.kind == kind {
            return true;
        }

        let mut source = self.source();

        while let Some(e) = source {
            match e.downcast_ref::<Error>() {
                Some(ref found) if found.kind == kind => return true,
                _ => source = e.source(),
            }
        }

        false
    }

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref source) = self.source {
            write!(f, "{}: {}", self.kind, source)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|cause| &**cause as &(dyn StdError + 'static))
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
