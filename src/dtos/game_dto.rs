use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

#[derive(Debug, sqlx::FromRow)]
pub struct GameDto {
    pub id: String,
    pub summoner_id: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub game_created_at: i64,
    pub assists: i64,
    pub deaths: i64,
    pub kills: i64,
    pub win: bool,
    pub notified: bool,
    pub champion_name: String,
    pub game_mode: String,
    pub lp_change: Option<i64>,
    pub promotion_text: Option<String>,
}

impl GameDto {
    pub async fn insert_or_ignore(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO game (
                id,
                summoner_id,
                game_created_at,
                assists,
                deaths,
                kills,
                win,
                lp_change,
                champion_name,
                game_mode,
                promotion_text
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.assists,
            self.deaths,
            self.kills,
            self.win,
            self.lp_change,
            self.champion_name,
            self.game_mode,
            self.promotion_text
        )
        .execute(pool)
        .await
        .context("failed to insert game")?;

        Ok(())
    }

    pub async fn upsert(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO game (
                id,
                summoner_id,
                game_created_at,
                assists,
                deaths,
                kills,
                win,
                lp_change,
                champion_name,
                game_mode,
                promotion_text
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.assists,
            self.deaths,
            self.kills,
            self.win,
            self.lp_change,
            self.champion_name,
            self.game_mode,
            self.promotion_text
        )
        .execute(pool)
        .await
        .context("failed to upsert game")?;

        Ok(())
    }

    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<GameDto>> {
        let games = sqlx::query_as!(GameDto, "SELECT * FROM game")
            .fetch_all(pool)
            .await
            .context("failed to query game")?;

        Ok(games)
    }
}
