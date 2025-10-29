use thiserror::Error;

pub type SdkResult<T> = Result<T, SdkError>;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
