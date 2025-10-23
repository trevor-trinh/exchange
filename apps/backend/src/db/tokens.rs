use crate::db::Db;
use crate::errors::Result;
use crate::models::{db::TokenRow, domain::Token};

impl Db {
    /// Create a new token
    pub async fn create_token(&self, ticker: String, decimals: u8, name: String) -> Result<Token> {
        let row = sqlx::query_as!(
            TokenRow,
            "INSERT INTO tokens (ticker, decimals, name) VALUES ($1, $2, $3)
             ON CONFLICT (ticker) DO UPDATE SET decimals = $2, name = $3
             RETURNING ticker, decimals, name",
            ticker,
            decimals as i32,
            name
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(Token {
            ticker: row.ticker,
            decimals: row.decimals as u8,
            name: row.name,
        })
    }

    /// Get a token by ticker
    pub async fn get_token(&self, ticker: &str) -> Result<Token> {
        let row = sqlx::query_as!(
            TokenRow,
            "SELECT ticker, decimals, name FROM tokens WHERE ticker = $1",
            ticker
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(Token {
            ticker: row.ticker,
            decimals: row.decimals as u8,
            name: row.name,
        })
    }

    /// List all tokens
    pub async fn list_tokens(&self) -> Result<Vec<Token>> {
        let rows = sqlx::query_as!(
            TokenRow,
            "SELECT ticker, decimals, name FROM tokens ORDER BY ticker",
        )
        .fetch_all(&self.postgres)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Token {
                ticker: row.ticker,
                decimals: row.decimals as u8,
                name: row.name,
            })
            .collect())
    }
}
