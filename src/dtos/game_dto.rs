use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use sqlx::{Pool, Sqlite};

pub struct GameDto {
    pub id: String,
    pub summoner_id: String,
    pub created_at: NaiveDateTime,
    pub champion_id: i64,
    pub assists: i64,
    pub deaths: i64,
    pub kills: i64,
    pub result: String,
    pub division: Option<i64>,
    pub lp: Option<i64>,
    pub tier: Option<String>,
    pub border_image_url: Option<String>,
    pub tier_image_url: Option<String>,
}

impl GameDto {
    pub async fn create(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO game (
                id,
                summoner_id,
                created_at,
                champion_id,
                assists,
                deaths,
                kills,
                result,
                division,
                lp,
                tier,
                border_image_url,
                tier_image_url
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.created_at,
            self.champion_id,
            self.assists,
            self.deaths,
            self.kills,
            self.result,
            self.division,
            self.lp,
            self.tier,
            self.border_image_url,
            self.tier_image_url
        )
        .execute(pool)
        .await
        .context("failed to insert game")?;

        Ok(())
    }
}
