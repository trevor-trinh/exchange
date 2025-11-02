use crate::db::Db;
use crate::errors::Result;
use crate::models::domain::Trade;
use sqlx::Row;

impl Db {
    /// Insert a new trade into the database
    pub async fn create_trade(&self, trade: &Trade) -> Result<()> {
        let price_str = trade.price.to_string();
        let size_str = trade.size.to_string();
        let side_str = match trade.side {
            crate::models::domain::Side::Buy => "buy",
            crate::models::domain::Side::Sell => "sell",
        };

        sqlx::query(
            r#"
            INSERT INTO trades (id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price, size, side, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9::side, $10)
            "#
        )
        .bind(trade.id)
        .bind(&trade.market_id)
        .bind(&trade.buyer_address)
        .bind(&trade.seller_address)
        .bind(trade.buyer_order_id)
        .bind(trade.seller_order_id)
        .bind(price_str)
        .bind(size_str)
        .bind(side_str)
        .bind(trade.timestamp)
        .execute(&self.postgres)
        .await?;

        Ok(())
    }

    /// Insert a new trade into the database (within a transaction)
    pub async fn create_trade_tx(
        &self,
        tx: &mut crate::db::Transaction<'_, crate::db::Postgres>,
        trade: &Trade,
    ) -> Result<()> {
        let price_str = trade.price.to_string();
        let size_str = trade.size.to_string();
        let side_str = match trade.side {
            crate::models::domain::Side::Buy => "buy",
            crate::models::domain::Side::Sell => "sell",
        };

        sqlx::query(
            r#"
            INSERT INTO trades (id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price, size, side, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9::side, $10)
            "#
        )
        .bind(trade.id)
        .bind(&trade.market_id)
        .bind(&trade.buyer_address)
        .bind(&trade.seller_address)
        .bind(trade.buyer_order_id)
        .bind(trade.seller_order_id)
        .bind(price_str)
        .bind(size_str)
        .bind(side_str)
        .bind(trade.timestamp)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_user_trades(
        &self,
        user_address: &str,
        market_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Trade>> {
        let limit = std::cmp::min(limit, 1000); // Cap at 1000

        let query = if let Some(market) = market_id {
            sqlx::query(
                r#"
                SELECT id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price::TEXT as price, size::TEXT as size, side, timestamp
                FROM trades
                WHERE (buyer_address = $1 OR seller_address = $1) AND market_id = $2
                ORDER BY timestamp DESC
                LIMIT $3
                "#
            )
            .bind(user_address)
            .bind(market)
            .bind(limit as i64)
        } else {
            sqlx::query(
                r#"
                SELECT id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price::TEXT as price, size::TEXT as size, side, timestamp
                FROM trades
                WHERE buyer_address = $1 OR seller_address = $1
                ORDER BY timestamp DESC
                LIMIT $2
                "#
            )
            .bind(user_address)
            .bind(limit as i64)
        };

        let rows = query.fetch_all(&self.postgres).await?;

        let trades = rows
            .iter()
            .map(|row| {
                let price_str: String = row.get("price");
                let size_str: String = row.get("size");
                let side_str: String = row.get("side");

                Trade {
                    id: row.get("id"),
                    market_id: row.get("market_id"),
                    buyer_address: row.get("buyer_address"),
                    seller_address: row.get("seller_address"),
                    buyer_order_id: row.get("buyer_order_id"),
                    seller_order_id: row.get("seller_order_id"),
                    price: price_str.parse().unwrap_or(0),
                    size: size_str.parse().unwrap_or(0),
                    side: if side_str == "buy" {
                        crate::models::domain::Side::Buy
                    } else {
                        crate::models::domain::Side::Sell
                    },
                    timestamp: row.get("timestamp"),
                }
            })
            .collect();

        Ok(trades)
    }

    pub async fn get_market_trades(&self, market_id: &str, limit: u32) -> Result<Vec<Trade>> {
        let limit = std::cmp::min(limit, 1000); // Cap at 1000

        let rows = sqlx::query(
            r#"
            SELECT id, market_id, buyer_address, seller_address, buyer_order_id, seller_order_id, price::TEXT as price, size::TEXT as size, side, timestamp
            FROM trades
            WHERE market_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#
        )
        .bind(market_id)
        .bind(limit as i64)
        .fetch_all(&self.postgres)
        .await?;

        let trades = rows
            .iter()
            .map(|row| {
                let price_str: String = row.get("price");
                let size_str: String = row.get("size");
                let side_str: String = row.get("side");

                Trade {
                    id: row.get("id"),
                    market_id: row.get("market_id"),
                    buyer_address: row.get("buyer_address"),
                    seller_address: row.get("seller_address"),
                    buyer_order_id: row.get("buyer_order_id"),
                    seller_order_id: row.get("seller_order_id"),
                    price: price_str.parse().unwrap_or(0),
                    size: size_str.parse().unwrap_or(0),
                    side: if side_str == "buy" {
                        crate::models::domain::Side::Buy
                    } else {
                        crate::models::domain::Side::Sell
                    },
                    timestamp: row.get("timestamp"),
                }
            })
            .collect();

        Ok(trades)
    }
}
