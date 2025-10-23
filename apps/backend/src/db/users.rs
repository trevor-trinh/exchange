use crate::db::Db;
use crate::models::{db::UserRow, domain::User};

impl Db {
    /// Create a new user
    pub async fn create_user(&self, address: String) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as!(
            UserRow,
            "INSERT INTO users (address) VALUES ($1) RETURNING address, created_at",
            address
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(User {
            address: row.address,
            created_at: row.created_at,
        })
    }

    /// Get a user by address
    pub async fn get_user(&self, address: &str) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as!(
            UserRow,
            "SELECT address, created_at FROM users WHERE address = $1",
            address
        )
        .fetch_one(&self.postgres)
        .await?;

        Ok(User {
            address: row.address,
            created_at: row.created_at,
        })
    }

    /// List all users
    pub async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let rows: Vec<UserRow> = sqlx::query_as!(
            UserRow,
            "SELECT address, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.postgres)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| User {
                address: row.address,
                created_at: row.created_at,
            })
            .collect())
    }
}
