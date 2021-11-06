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

impl<T, I> From<tokio_tower::Error<T, I>> for Error<<T as TryStream>::Error, <T as Sink<I>>::Error>
where
    T: Sink<I> + TryStream,
{
    fn from(error: tokio_tower::Error<T, I>) -> Self {
        use tokio_tower::Error::*;

        match error {
            BrokenTransportSend(e) => Self::Write(e),
            BrokenTransportRecv(Some(e)) => Self::Read(e),
            BrokenTransportRecv(None) | ClientDropped => Self::ConnectionDropped,
            TransportFull => Self::TransportFull,
            Desynchronized => Self::Desynchronized,
        }
    }
}

impl<R, W> Error<R, W> {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Api(e) if e.is_auth_error())
    }
}
