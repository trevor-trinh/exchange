use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::{Order, OrderStatus, OrderType, Side};
use crate::utils::BigDecimalExt;
use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

impl Db {
    /// Insert a new order into the database
    pub async fn create_order(&self, order: &Order) -> Result<()> {
        // For market orders, use price 1 in DB (actual price doesn't matter for market orders)
        let price_for_db = if order.order_type == OrderType::Market && order.price == 0 {
            1
        } else {
            order.price
        };

        let price_str = price_for_db.to_string();
        let size_str = order.size.to_string();
        let filled_size_str = order.filled_size.to_string();
        let side_str = order.side.to_string();
        let order_type_str = order.order_type.to_string();
        let status_str = order.status.to_string();

        sqlx::query(
            r#"
            INSERT INTO orders (id, user_address, market_id, price, size, side, type, status, filled_size, created_at, updated_at)
            VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::side, $7::order_type, $8::order_status, $9::numeric, $10, $11)
            "#
        )
        .bind(order.id)
        .bind(&order.user_address)
        .bind(&order.market_id)
        .bind(price_str)
        .bind(size_str)
        .bind(side_str)
        .bind(order_type_str)
        .bind(status_str)
        .bind(filled_size_str)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    /// Update an order's filled size and status
    pub async fn update_order_fill(
        &self,
        order_id: Uuid,
        filled_size: u128,
        status: OrderStatus,
    ) -> Result<()> {
        let filled_size_str = filled_size.to_string();
        let status_str = status.to_string();

        sqlx::query(
            r#"
            UPDATE orders
            SET filled_size = $1::numeric, status = $2::order_status, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(filled_size_str)
        .bind(status_str)
        .bind(Utc::now())
        .bind(order_id)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    /// Update an order's filled size and status (within a transaction)
    pub async fn update_order_fill_tx(
        &self,
        tx: &mut crate::db::Transaction<'_, crate::db::Postgres>,
        order_id: Uuid,
        filled_size: u128,
        status: OrderStatus,
    ) -> Result<()> {
        let filled_size_str = filled_size.to_string();
        let status_str = status.to_string();

        sqlx::query(
            r#"
            UPDATE orders
            SET filled_size = $1::numeric, status = $2::order_status, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(filled_size_str)
        .bind(status_str)
        .bind(Utc::now())
        .bind(order_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_order(&self, order_id: &Uuid) -> Result<Order> {
        let row = sqlx::query(
            r#"
            SELECT id, user_address, market_id, price, size, side::TEXT as side, type::TEXT as type, status::TEXT as status, filled_size, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#
        )
        .bind(order_id)
        .fetch_one(&self.postgres)
        .await?;

        let price: BigDecimal = row.get("price");
        let size: BigDecimal = row.get("size");
        let filled_size: BigDecimal = row.get("filled_size");
        let side_str: String = row.get("side");
        let type_str: String = row.get("type");
        let status_str: String = row.get("status");

        Ok(Order {
            id: row.get("id"),
            user_address: row.get("user_address"),
            market_id: row.get("market_id"),
            price: price.to_u128(),
            size: size.to_u128(),
            side: side_str.parse().unwrap_or(Side::Buy),
            order_type: type_str.parse().unwrap_or(OrderType::Limit),
            status: status_str.parse().unwrap_or(OrderStatus::Pending),
            filled_size: filled_size.to_u128(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn get_user_orders(
        &self,
        user_address: &str,
        market_id: Option<&str>,
        status: Option<OrderStatus>,
        limit: u32,
    ) -> Result<Vec<Order>> {
        let limit = std::cmp::min(limit, 1000); // Cap at 1000

        let status_str = status.map(|s| s.to_string());

        let query = if let (Some(market), Some(stat)) = (market_id, &status_str) {
            sqlx::query(
                r#"
                SELECT id, user_address, market_id, price, size, side::TEXT as side, type::TEXT as type, status::TEXT as status, filled_size, created_at, updated_at
                FROM orders
                WHERE user_address = $1 AND market_id = $2 AND status = $3::order_status
                ORDER BY created_at DESC
                LIMIT $4
                "#
            )
            .bind(user_address)
            .bind(market)
            .bind(stat)
            .bind(limit as i64)
        } else if let Some(market) = market_id {
            sqlx::query(
                r#"
                SELECT id, user_address, market_id, price, size, side::TEXT as side, type::TEXT as type, status::TEXT as status, filled_size, created_at, updated_at
                FROM orders
                WHERE user_address = $1 AND market_id = $2
                ORDER BY created_at DESC
                LIMIT $3
                "#
            )
            .bind(user_address)
            .bind(market)
            .bind(limit as i64)
        } else if let Some(stat) = &status_str {
            sqlx::query(
                r#"
                SELECT id, user_address, market_id, price, size, side::TEXT as side, type::TEXT as type, status::TEXT as status, filled_size, created_at, updated_at
                FROM orders
                WHERE user_address = $1 AND status = $2::order_status
                ORDER BY created_at DESC
                LIMIT $3
                "#
            )
            .bind(user_address)
            .bind(stat)
            .bind(limit as i64)
        } else {
            sqlx::query(
                r#"
                SELECT id, user_address, market_id, price, size, side::TEXT as side, type::TEXT as type, status::TEXT as status, filled_size, created_at, updated_at
                FROM orders
                WHERE user_address = $1
                ORDER BY created_at DESC
                LIMIT $2
                "#
            )
            .bind(user_address)
            .bind(limit as i64)
        };

        let rows = query.fetch_all(&self.postgres).await?;

        let orders = rows
            .iter()
            .map(|row| {
                let price: BigDecimal = row.get("price");
                let size: BigDecimal = row.get("size");
                let filled_size: BigDecimal = row.get("filled_size");
                let side_str: String = row.get("side");
                let type_str: String = row.get("type");
                let status_str: String = row.get("status");

                Order {
                    id: row.get("id"),
                    user_address: row.get("user_address"),
                    market_id: row.get("market_id"),
                    price: price.to_u128(),
                    size: size.to_u128(),
                    side: side_str.parse().unwrap_or(Side::Buy),
                    order_type: type_str.parse().unwrap_or(OrderType::Limit),
                    status: status_str.parse().unwrap_or(OrderStatus::Pending),
                    filled_size: filled_size.to_u128(),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect();

        Ok(orders)
    }
}
