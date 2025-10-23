use sqlx::PgPool;

use crate::models::domain::Balance;

// TODO: Implement balance operations
// - get_balance(pool: &PgPool, user_address: &str, token_ticker: &str) -> Result<Balance, sqlx::Error>
// - list_balances_by_user(pool: &PgPool, user_address: &str) -> Result<Vec<Balance>, sqlx::Error>
// - update_balance(pool: &PgPool, user_address: &str, token_ticker: &str, amount: u128) -> Result<Balance, sqlx::Error>
// - lock_balance(pool: &PgPool, user_address: &str, token_ticker: &str, amount: u128) -> Result<Balance, sqlx::Error>
// - unlock_balance(pool: &PgPool, user_address: &str, token_ticker: &str, amount: u128) -> Result<Balance, sqlx::Error>
