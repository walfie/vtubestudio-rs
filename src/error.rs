use crate::data::{ApiError, ResponseData};

use futures_core::TryStream;
use futures_sink::Sink;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<R, W> {
    #[error("underlying transport failed while attempting to receive a response")]
    Read(R),
    #[error("underlying transport failed to send a request")]
    Write(W),
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
    #[error("no more in-flight requests allowed")]
    TransportFull,
    #[error("connection was dropped")]
    ConnectionDropped,
    #[error("received server response with unexpected request ID")]
    Desynchronized,
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received:?})")]
    UnexpectedResponse {
        expected: &'static str,
        received: ResponseData,
    },
}

#[derive(Error, Debug)]
pub enum TransportError<E> {
    #[error("underlying transport failed")]
    Underlying(E),
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
}

impl<T, I, R, W> From<tokio_tower::Error<T, I>> for Error<R, W>
where
    T: Sink<I, Error = TransportError<W>> + TryStream<Error = TransportError<R>>,
{
    fn from(error: tokio_tower::Error<T, I>) -> Self {
        use tokio_tower::Error::*;

        match error {
            BrokenTransportSend(TransportError::Underlying(e)) => Error::Write(e),
            BrokenTransportSend(TransportError::Json(e)) => Error::Json(e),
            BrokenTransportRecv(Some(TransportError::Underlying(e))) => Error::Read(e),
            BrokenTransportRecv(Some(TransportError::Json(e))) => Error::Json(e),
            BrokenTransportRecv(None) | ClientDropped => Error::ConnectionDropped,
            TransportFull => Error::TransportFull,
            Desynchronized => Error::Desynchronized,
        }
    }
}

impl<R, W> Error<R, W> {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Api(e) if e.is_auth_error())
    }
}
