use crate::client::IdTagger;
use crate::data::{ApiError, RequestEnvelope, ResponseData};
use crate::transport::{ApiTransport, WebSocketTransport};
use tokio_tower::multiplex::MultiplexTransport;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<T: WebSocketTransport> {
    #[error("transport error")]
    Transport(#[from] TransportError<T::StreamError, T::SinkError>),
    #[error("transport error")]
    Multiplex(
        #[from] tokio_tower::Error<MultiplexTransport<ApiTransport<T>, IdTagger>, RequestEnvelope>,
    ),
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received:?})")]
    UnexpectedResponse {
        expected: &'static str,
        received: ResponseData,
    },
}

#[derive(Error, Debug)]
pub enum TransportError<R, W> {
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
    #[error("read error")]
    Read(R),
    #[error("write error")]
    Write(W),
}

impl<T: WebSocketTransport> Error<T> {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Api(e) if e.is_auth_error())
    }
}
