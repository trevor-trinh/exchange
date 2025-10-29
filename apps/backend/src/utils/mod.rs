use axum::http::StatusCode;
use axum::Json;
use bigdecimal::num_bigint::ToBigInt;
use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// Trait for converting between BigDecimal and u128
pub trait BigDecimalExt {
    fn to_u128(self) -> u128;
    fn from_u128(value: u128) -> Self;
}

impl BigDecimalExt for BigDecimal {
    fn to_u128(self) -> u128 {
        // Convert to BigInt - this will panic if there's a fractional part,
        // which is correct since we should NEVER have fractions (everything is in atoms)
        let bigint = self
            .to_bigint()
            .expect("BUG: BigDecimal has fractional part - all values should be in atoms");

        // Convert BigInt to u128
        bigint.to_u128().expect("BUG: Value out of u128 range")
    }

    fn from_u128(value: u128) -> Self {
        BigDecimal::from(value)
    }
}

/// Parse a u128 parameter from a string with proper error handling for REST APIs
pub fn parse_u128_param(
    s: &str,
    param_name: &str,
) -> Result<u128, (StatusCode, Json<ErrorResponse>)> {
    s.parse::<u128>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid {} format: must be a valid u128", param_name),
                code: format!("INVALID_{}", param_name.to_uppercase()),
            }),
        )
    })
}
