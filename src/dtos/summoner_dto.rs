use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

use crate::dtos::game_dto::GameDto;

use super::guild_dto::GuildDto;

#[derive(Debug)]
pub struct SummonerDto {
    pub id: String,
    pub name: String,
    pub guild_id: i64,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub queue_type: Option<String>,
    pub tier: Option<String>,
    pub lp: Option<i64>,
    pub division: Option<String>,
}

impl SummonerDto {
    pub async fn insert_or_ignore(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO summoner (
                id,
                name,
                guild_id,
                queue_type,
                tier,
                lp,
                division
                )
            VALUES (?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.name,
            self.guild_id,
            self.queue_type,
            self.tier,
            self.lp,
            self.division,
        )
        .execute(pool)
        .await
        .context("summoner_dto: failed to insert summoner")?;

        Ok(())
    }

    pub async fn upsert(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO summoner (
                id,
                name,
                guild_id,
                queue_type,
                tier,
                lp,
                division
                )
            VALUES (?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.name,
            self.guild_id,
            self.queue_type,
            self.tier,
            self.lp,
            self.division,
        )
        .execute(pool)
        .await
        .context("summoner_dto: failed to upsert summoner")?;

        Ok(())
    }

    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<SummonerDto>> {
        let summoners = sqlx::query_as!(SummonerDto, "SELECT * FROM summoner")
            .fetch_all(pool)
            .await
            .context("summoner_dto: failed to query summoners")?;

        Ok(summoners)
    }

    pub async fn get(pool: &Pool<Sqlite>, summoner_id: &str) -> Result<SummonerDto> {
        let summoner = sqlx::query_as!(
            SummonerDto,
            r#"
            SELECT * FROM summoner
            WHERE id = ?;
            "#,
            summoner_id
        )
        .fetch_one(pool)
        .await
        .context("summoner_dto: failed to query summoner")?;

        Ok(summoner)
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
        .context("summoner_dto: failed to delete summoner")?;

        Ok(())
    }

    pub async fn get_guild(&self, pool: &Pool<Sqlite>) -> Result<GuildDto> {
        let guild = sqlx::query_as!(
            GuildDto,
            r#"
            SELECT * FROM guild
            WHERE id = ?;
            "#,
            self.guild_id
        )
        .fetch_one(pool)
        .await
        .context("summoner_dto: failed to query guild")?;

        Ok(guild)
    }
}
