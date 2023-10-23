use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

pub struct SummonerDto {
    pub id: String,
    pub name: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl SummonerDto {
    pub async fn create(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO summoner (
                id,
                name
                )
            VALUES (?, ?);
            "#,
            self.id,
            self.name,
        )
        .execute(pool)
        .await
        .context("failed to insert summoner")?;

        Ok(())
    }

    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<SummonerDto>> {
        let summoners = sqlx::query_as!(SummonerDto, "SELECT * FROM summoner")
            .fetch_all(pool)
            .await
            .context("failed to query summoners")?;

        Ok(summoners)
    }
}
