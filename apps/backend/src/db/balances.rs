use crate::db::Db;
use crate::errors::Result;
use crate::models::db::BalanceRow;
use crate::models::domain::Balance;
use chrono::Utc;

impl Db {
    /// Get balance for a specific user and token
    pub async fn get_balance(&self, user_address: &str, token_ticker: &str) -> Result<Balance> {
        let row: BalanceRow = sqlx::query_as(
            r#"
            SELECT user_address, token_ticker, amount, open_interest, updated_at
            FROM balances
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .fetch_one(&self.postgres)
        .await?;

        Ok(row.into())
    }

    /// List all balances for a user
    pub async fn list_balances_by_user(&self, user_address: &str) -> Result<Vec<Balance>> {
        let rows: Vec<BalanceRow> = sqlx::query_as(
            r#"
            SELECT user_address, token_ticker, amount, open_interest, updated_at
            FROM balances
            WHERE user_address = $1
            "#,
        )
        .bind(user_address)
        .fetch_all(&self.postgres)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    /// Update or insert balance (upsert)
    pub async fn update_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<Balance> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO balances (user_address, token_ticker, amount, open_interest, updated_at)
            VALUES ($1, $2, $3::numeric, 0, $4)
            ON CONFLICT (user_address, token_ticker)
            DO UPDATE SET
                amount = $3::numeric,
                updated_at = $4
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        Ok(Balance {
            user_address: user_address.to_string(),
            token_ticker: token_ticker.to_string(),
            amount,
            open_interest: 0,
            updated_at: now,
        })
    }

    /// Add to existing balance (for deposits/credits)
    pub async fn add_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount_delta: u128,
    ) -> Result<Balance> {
        let delta_str = amount_delta.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO balances (user_address, token_ticker, amount, open_interest, updated_at)
            VALUES ($1, $2, $3::numeric, 0, $4)
            ON CONFLICT (user_address, token_ticker)
            DO UPDATE SET
                amount = balances.amount + $3::numeric,
                updated_at = $4
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&delta_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        self.get_balance(user_address, token_ticker).await
    }

    /// Subtract from existing balance (for withdrawals/debits)
    pub async fn subtract_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount_delta: u128,
    ) -> Result<Balance> {
        let delta_str = amount_delta.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE balances
            SET amount = amount - $3::numeric, updated_at = $4
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&delta_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        self.get_balance(user_address, token_ticker).await
    }

    /// Lock funds in open_interest (when placing an order)
    /// Returns error if insufficient available balance
    pub async fn lock_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<Balance> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        // Check if user has sufficient available balance (amount - open_interest >= amount to lock)
        let result = sqlx::query(
            r#"
            UPDATE balances
            SET open_interest = open_interest + $3::numeric, updated_at = $4
            WHERE user_address = $1
              AND token_ticker = $2
              AND amount - open_interest >= $3::numeric
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::errors::ExchangeError::InsufficientBalance {
                user_address: user_address.to_string(),
                token_ticker: token_ticker.to_string(),
                required: amount,
            });
        }

        self.get_balance(user_address, token_ticker).await
    }

    /// Unlock funds from open_interest (when cancelling/filling an order)
    /// If balance doesn't exist, this is a no-op (nothing to unlock)
    pub async fn unlock_balance(
        &self,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<Balance> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE balances
            SET open_interest = open_interest - $3::numeric, updated_at = $4
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&self.postgres)
        .await?;

        // If balance doesn't exist (0 rows updated), create a zero balance record
        if result.rows_affected() == 0 {
            return Ok(Balance {
                user_address: user_address.to_string(),
                token_ticker: token_ticker.to_string(),
                amount: 0,
                open_interest: 0,
                updated_at: now,
            });
        }

        self.get_balance(user_address, token_ticker).await
    }

    /// Lock balance within a transaction (for atomic operations)
    pub async fn lock_balance_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<()> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE balances
            SET open_interest = open_interest + $3::numeric, updated_at = $4
            WHERE user_address = $1
              AND token_ticker = $2
              AND amount - open_interest >= $3::numeric
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::errors::ExchangeError::InsufficientBalance {
                user_address: user_address.to_string(),
                token_ticker: token_ticker.to_string(),
                required: amount,
            });
        }

        Ok(())
    }

    /// Unlock balance within a transaction (for atomic operations)
    pub async fn unlock_balance_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<()> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE balances
            SET open_interest = open_interest - $3::numeric, updated_at = $4
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Add balance within a transaction (for atomic operations)
    pub async fn add_balance_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<()> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO balances (user_address, token_ticker, amount, open_interest, updated_at)
            VALUES ($1, $2, $3::numeric, 0, $4)
            ON CONFLICT (user_address, token_ticker)
            DO UPDATE SET
                amount = balances.amount + $3::numeric,
                updated_at = $4
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Subtract balance within a transaction (for atomic operations)
    pub async fn subtract_balance_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_address: &str,
        token_ticker: &str,
        amount: u128,
    ) -> Result<()> {
        let amount_str = amount.to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE balances
            SET amount = amount - $3::numeric, updated_at = $4
            WHERE user_address = $1 AND token_ticker = $2
            "#,
        )
        .bind(user_address)
        .bind(token_ticker)
        .bind(&amount_str)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
