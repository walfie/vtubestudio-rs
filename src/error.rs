use crate::data::{ApiError, ResponseData};
use crate::transport::Transport;

#[derive(thiserror::Error, Debug)]
pub enum Error<T: Transport> {
    #[error("transport error")]
    Transport(#[from] tokio_tower::Error<T::Underlying, T::Message>),
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received:?})")]
    UnexpectedResponse {
        expected: &'static str,
        received: ResponseData,
    },
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
    #[error("unexpected websocket message")]
    UnexpectedWebSocketMessage(T::Message),
}

impl<T: Transport> Error<T> {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Api(e) if e.is_auth_error())
    }
}
