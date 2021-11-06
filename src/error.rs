use crate::data::{ApiError, ResponseData};

use futures_core::TryStream;
use futures_sink::Sink;
use thiserror::Error;
use tokio_tungstenite::tungstenite;

#[derive(Error, Debug, PartialEq)]
pub enum Error<T> {
    #[error("transport error")]
    Transport(T),
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received:?})")]
    UnexpectedResponse {
        expected: &'static str,
        received: ResponseData,
    },
}

/// Type alias for a [TransportError] where the read and write error types are the same
pub type UnifiedTransportError<E> = TransportError<E, E>;

#[derive(Error, Debug, PartialEq)]
pub enum TransportError<R, W> {
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
}

#[derive(Error, Debug)]
pub enum WebSocketError<E> {
    #[error("underlying websocket transport failed")]
    Underlying(E),
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
}

pub type TungsteniteError = Error<TungsteniteTransportError>;
pub type TungsteniteTransportError = TransportError<TungsteniteWsError, TungsteniteWsError>;
pub type TungsteniteWsError = WebSocketError<tungstenite::Error>;

impl<T, I> From<tokio_tower::Error<T, I>>
    for TransportError<<T as TryStream>::Error, <T as Sink<I>>::Error>
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

impl<T> Error<T> {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Api(e) if e.is_auth_error())
    }
}
