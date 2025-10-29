use crate::db::Db;
use crate::errors::Result;
use crate::models::{db::UserRow, domain::User};

impl Db {
    /// Create a new user
    pub async fn create_user(&self, address: String) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            "INSERT INTO users (address) VALUES ($1) RETURNING address, created_at",
            address
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(row.into())
    }

    /// Get a user by address
    pub async fn get_user(&self, address: &str) -> Result<User> {
        let row: UserRow = sqlx::query_as!(
            UserRow,
            "SELECT address, created_at FROM users WHERE address = $1",
            address
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(row.into())
    }

    /// List all users
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let rows: Vec<UserRow> = sqlx::query_as!(
            UserRow,
            "SELECT address, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.postgres)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }
}
