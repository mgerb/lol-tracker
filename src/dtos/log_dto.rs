use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

pub struct LogDto {
    pub id: Option<i64>,
    pub message: String,
    pub error_type: ErrorType,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Info,
    Error,
}

impl Into<String> for ErrorType {
    fn into(self) -> String {
        match self {
            ErrorType::Info => "info".to_string(),
            ErrorType::Error => "error".to_string(),
        }
    }
}

impl Into<ErrorType> for String {
    fn into(self) -> ErrorType {
        match self.as_str() {
            "info" => ErrorType::Info,
            "error" => ErrorType::Error,
            _ => panic!("invalid error type"),
        }
    }
}

impl LogDto {
    pub fn new(message: &str, error_type: ErrorType) -> Self {
        Self {
            id: None,
            message: message.to_string(),
            error_type,
            created_at: None,
        }
    }

    pub async fn info(pool: &Pool<Sqlite>, message: &str) -> Result<()> {
        Self::new(message, ErrorType::Info).create(pool).await?;
        Ok(())
    }

    pub async fn error(pool: &Pool<Sqlite>, message: &str) -> Result<()> {
        Self::new(message, ErrorType::Error).create(pool).await?;
        Ok(())
    }

    pub async fn create(&self, pool: &Pool<Sqlite>) -> Result<()> {
        let error_type: String = self.error_type.into();
        sqlx::query!(
            r#"
            INSERT INTO log (
               message,
               error_type
                )
            VALUES (?, ?);
            "#,
            self.message,
            error_type
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
