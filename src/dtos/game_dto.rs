use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

pub struct GameDto {
    pub id: String,
    pub summoner_id: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub game_created_at: i64,
    pub champion_id: i64,
    pub assists: i64,
    pub deaths: i64,
    pub kills: i64,
    pub result: String,
    pub notified: bool,
    pub division: Option<i64>,
    pub lp: Option<i64>,
    pub tier: Option<String>,
    pub border_image_url: Option<String>,
    pub tier_image_url: Option<String>,
}

impl GameDto {
    pub async fn insert_or_ignore(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO game (
                id,
                summoner_id,
                game_created_at,
                champion_id,
                assists,
                deaths,
                kills,
                result,
                division,
                lp,
                tier,
                border_image_url,
                tier_image_url,
                notified
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.champion_id,
            self.assists,
            self.deaths,
            self.kills,
            self.result,
            self.division,
            self.lp,
            self.tier,
            self.border_image_url,
            self.tier_image_url,
            self.notified
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
                champion_id,
                assists,
                deaths,
                kills,
                result,
                division,
                lp,
                tier,
                border_image_url,
                tier_image_url,
                notified
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.champion_id,
            self.assists,
            self.deaths,
            self.kills,
            self.result,
            self.division,
            self.lp,
            self.tier,
            self.border_image_url,
            self.tier_image_url,
            self.notified
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
