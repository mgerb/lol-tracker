use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

pub struct LogDto {
    pub id: i64,
    pub message: String,
    pub created_at: Option<i64>,
}

impl LogDto {
    pub async fn create(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO log (
               message 
                )
            VALUES (?);
            "#,
            self.message,
        )
        .execute(pool)
        .await
        .context("failed to insert log")?;

        Ok(())
    }

    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<LogDto>> {
        let logs = sqlx::query_as!(LogDto, "SELECT * FROM log")
            .fetch_all(pool)
            .await
            .context("failed to query log")?;

        Ok(logs)
    }
}
