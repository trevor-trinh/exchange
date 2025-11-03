use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Error, Debug)]
pub enum ExchangeError {
    // Business logic errors (4xx)
    #[error("Token '{ticker}' does not exist")]
    TokenNotFound { ticker: String },

    #[error("Market '{market_id}' does not exist")]
    MarketNotFound { market_id: String },

    #[error("Market '{market_id}' already exists")]
    MarketAlreadyExists { market_id: String },

    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    #[error("Invalid price format")]
    InvalidPrice,

    #[error("Invalid size format")]
    InvalidSize,

    #[error("Invalid amount format")]
    InvalidAmount,

    #[error("Order value overflow or division error")]
    OrderValueOverflow,

    #[error("Price does not meet tick size requirement")]
    InvalidTickSize,

    #[error("Size does not meet lot size requirement")]
    InvalidLotSize,

    #[error("Size is below minimum required")]
    SizeBelowMinimum,

    #[error("Insufficient balance for user '{user_address}' token '{token_ticker}': required {required}")]
    InsufficientBalance {
        user_address: String,
        token_ticker: String,
        required: u128,
    },

    #[error("Order not found")]
    OrderNotFound,

    #[error("User '{address}' not found")]
    UserNotFound { address: String },

    #[error("Failed to send order to engine")]
    EngineSendFailed,

    #[error("Failed to receive response from engine")]
    EngineReceiveFailed,

    #[error("Failed to unlock balance")]
    UnlockFailed,

    // Infrastructure errors (5xx) - auto-converted
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("ClickHouse error: {0}")]
    ClickHouse(#[from] clickhouse::error::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("UUID parse error: {0}")]
    UuidParseError(#[from] uuid::Error),
}

pub type Result<T> = std::result::Result<T, ExchangeError>;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl ExchangeError {
    /// Get the error code for this error
    fn error_code(&self) -> &'static str {
        match self {
            ExchangeError::TokenNotFound { .. } => "TOKEN_NOT_FOUND",
            ExchangeError::MarketNotFound { .. } => "MARKET_NOT_FOUND",
            ExchangeError::MarketAlreadyExists { .. } => "MARKET_ALREADY_EXISTS",
            ExchangeError::InvalidParameter { .. } => "INVALID_PARAMETER",
            ExchangeError::InvalidPrice => "INVALID_PRICE",
            ExchangeError::InvalidSize => "INVALID_SIZE",
            ExchangeError::InvalidAmount => "INVALID_AMOUNT",
            ExchangeError::OrderValueOverflow => "ORDER_VALUE_OVERFLOW",
            ExchangeError::InvalidTickSize => "INVALID_TICK_SIZE",
            ExchangeError::InvalidLotSize => "INVALID_LOT_SIZE",
            ExchangeError::SizeBelowMinimum => "SIZE_BELOW_MINIMUM",
            ExchangeError::InsufficientBalance { .. } => "INSUFFICIENT_BALANCE",
            ExchangeError::OrderNotFound => "ORDER_NOT_FOUND",
            ExchangeError::UserNotFound { .. } => "USER_NOT_FOUND",
            ExchangeError::EngineSendFailed => "ENGINE_SEND_FAILED",
            ExchangeError::EngineReceiveFailed => "ENGINE_RECEIVE_FAILED",
            ExchangeError::UnlockFailed => "UNLOCK_FAILED",
            ExchangeError::Database(_) => "DATABASE_ERROR",
            ExchangeError::ClickHouse(_) => "CLICKHOUSE_ERROR",
            ExchangeError::ParseError(_) => "PARSE_ERROR",
            ExchangeError::UuidParseError(_) => "UUID_PARSE_ERROR",
        }
    }

    /// Get the HTTP status code for this error
    fn status_code(&self) -> StatusCode {
        match self {
            // Client errors
            ExchangeError::TokenNotFound { .. } => StatusCode::NOT_FOUND,
            ExchangeError::MarketNotFound { .. } => StatusCode::NOT_FOUND,
            ExchangeError::OrderNotFound => StatusCode::NOT_FOUND,
            ExchangeError::UserNotFound { .. } => StatusCode::NOT_FOUND,
            ExchangeError::MarketAlreadyExists { .. } => StatusCode::CONFLICT,
            ExchangeError::InvalidParameter { .. } => StatusCode::BAD_REQUEST,
            ExchangeError::InvalidPrice => StatusCode::BAD_REQUEST,
            ExchangeError::InvalidSize => StatusCode::BAD_REQUEST,
            ExchangeError::InvalidAmount => StatusCode::BAD_REQUEST,
            ExchangeError::OrderValueOverflow => StatusCode::BAD_REQUEST,
            ExchangeError::InvalidTickSize => StatusCode::BAD_REQUEST,
            ExchangeError::InvalidLotSize => StatusCode::BAD_REQUEST,
            ExchangeError::SizeBelowMinimum => StatusCode::BAD_REQUEST,
            ExchangeError::InsufficientBalance { .. } => StatusCode::BAD_REQUEST,
            ExchangeError::ParseError(_) => StatusCode::BAD_REQUEST,
            ExchangeError::UuidParseError(_) => StatusCode::BAD_REQUEST,
            // Server errors
            ExchangeError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ExchangeError::ClickHouse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ExchangeError::EngineSendFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ExchangeError::EngineReceiveFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ExchangeError::UnlockFailed => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ExchangeError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();

        // For server errors, log the details but don't expose them
        let error_message = match &self {
            ExchangeError::Database(ref e) => {
                log::error!("Database error: {}", e);
                "Internal server error".to_string()
            }
            ExchangeError::ClickHouse(ref e) => {
                log::error!("ClickHouse error: {}", e);
                "Internal server error".to_string()
            }
            _ => self.to_string(),
        };

        let body = Json(ErrorResponse {
            error: error_message,
            code: error_code.to_string(),
        });

        (status, body).into_response()
    }
}
