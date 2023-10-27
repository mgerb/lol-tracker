use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

#[derive(Debug)]
pub struct SummonerDto {
    pub id: String,
    pub name: String,
    pub guild_id: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl SummonerDto {
    pub async fn create(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO summoner (
                id,
                name,
                guild_id
                )
            VALUES (?, ?, ?);
            "#,
            self.id,
            self.name,
            self.guild_id
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

    pub async fn delete(pool: &Pool<Sqlite>, summoner_name: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM game
            WHERE summoner_id = (SELECT id FROM summoner WHERE name = ?);

            DELETE FROM summoner
            WHERE name = ?;
            "#,
            summoner_name,
            summoner_name,
        )
        .execute(pool)
        .await
        .context("failed to delete summoner")?;

        Ok(())
    }
}
