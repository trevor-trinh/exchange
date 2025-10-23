use sqlx::PgPool;
use uuid::Uuid;

use crate::models::domain::Trade;

// TODO: Implement trade operations
// - create_trade(pool: &PgPool, ...) -> Result<Trade, sqlx::Error>
// - get_trade(pool: &PgPool, id: Uuid) -> Result<Trade, sqlx::Error>
// - list_trades_by_market(pool: &PgPool, market_id: &str) -> Result<Vec<Trade>, sqlx::Error>
// - list_trades_by_user(pool: &PgPool, user_address: &str) -> Result<Vec<Trade>, sqlx::Error>
// - get_recent_trades(pool: &PgPool, market_id: &str, limit: i64) -> Result<Vec<Trade>, sqlx::Error>

impl Db {}
