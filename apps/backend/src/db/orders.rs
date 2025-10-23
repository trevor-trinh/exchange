use sqlx::PgPool;
use uuid::Uuid;

use crate::models::domain::{Order, OrderStatus};

// TODO: Implement order operations
// - create_order(pool: &PgPool, ...) -> Result<Order, sqlx::Error>
// - get_order(pool: &PgPool, id: Uuid) -> Result<Order, sqlx::Error>
// - list_orders_by_user(pool: &PgPool, user_address: &str) -> Result<Vec<Order>, sqlx::Error>
// - list_orders_by_market(pool: &PgPool, market_id: &str) -> Result<Vec<Order>, sqlx::Error>
// - list_orders_by_status(pool: &PgPool, status: OrderStatus) -> Result<Vec<Order>, sqlx::Error>
// - update_order_status(pool: &PgPool, id: Uuid, status: OrderStatus, filled_size: u128) -> Result<Order, sqlx::Error>
// - cancel_order(pool: &PgPool, id: Uuid) -> Result<Order, sqlx::Error>
