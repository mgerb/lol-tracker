use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub async fn create_db() -> Result<Pool<Sqlite>> {
    std::fs::File::create("db.sqlite").context("failed to create db.sqlite")?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:db.sqlite")
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
