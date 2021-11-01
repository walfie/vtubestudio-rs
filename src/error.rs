use crate::data::{ApiError, ResponseData};
use std::error::Error as StdError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("transport error")]
    Transport(#[from] Box<dyn StdError + Send>),
    #[error("received APIError {}: {}", .0.error_id, .0.message)]
    Api(ApiError),
    #[error("received unexpected response (expected {expected}, received {received:?})")]
    UnexpectedResponse {
        expected: &'static str,
        received: ResponseData,
    },
    #[error("failed to parse JSON")]
    Json(#[from] serde_json::Error),
    #[error("websocket error")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::error::Error),
    #[error("unexpected websocket message: {0}")]
    UnexpectedWebSocketMessage(tokio_tungstenite::tungstenite::Message),
}

impl Error {
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Api(e) if e.is_auth_error())
    }
}
