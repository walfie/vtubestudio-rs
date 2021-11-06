use futures_core::TryStream;
use futures_sink::Sink;
use std::error::Error as StdError;
use std::fmt;

pub type BoxError = Box<dyn StdError + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Option<BoxError>,
}

#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error("JSON error")]
    Json,
    #[error("websocket error")]
    WebSocket,
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
    #[error("custom error")]
    Custom,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error { kind, source: None }
    }

    pub fn new_with_source<E: Into<BoxError>>(kind: ErrorKind, source: Option<E>) -> Self {
        Error {
            kind,
            source: source.map(Into::into),
        }
    }

    pub fn with_source<E: Into<BoxError>>(mut self, source: E) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn new_websocket<E: Into<BoxError>>(source: E) -> Self {
        Self::new(ErrorKind::WebSocket).with_source(source)
    }

    pub fn new_read<E: Into<BoxError>>(source: E) -> Self {
        Self::new(ErrorKind::Read).with_source(source)
    }

    pub fn new_json<E: Into<BoxError>>(source: E) -> Self {
        Self::new(ErrorKind::Json).with_source(source)
    }

    pub fn new_write<E: Into<BoxError>>(source: E) -> Self {
        Self::new(ErrorKind::Write).with_source(source)
    }

    pub fn new_custom<E: Into<BoxError>>(source: E) -> Self {
        Self::new(ErrorKind::Custom).with_source(source)
    }

    pub fn find_source<E: StdError + 'static>(&self) -> Option<&E> {
        let mut source = self.source();

        while let Some(e) = source {
            match e.downcast_ref() {
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
            BrokenTransportSend(e) => Self::new_write(e),
            BrokenTransportRecv(Some(e)) => Self::new_read(e),
            BrokenTransportRecv(None) | ClientDropped => Self::new(ErrorKind::ConnectionDropped),
            TransportFull => Self::new(ErrorKind::TransportFull),
            Desynchronized => Self::new(ErrorKind::Desynchronized),
        }
    }
}
