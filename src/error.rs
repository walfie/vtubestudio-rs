use crate::data::{ApiError, ResponseData};
use std::error::Error as StdError;

pub type Result<T> = std::result::Result<T, Error>;

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
}
