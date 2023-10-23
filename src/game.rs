use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

#[derive(Serialize, Deserialize)]
pub struct Game {
    id: String,
    myData: MyData,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Into<GameDto> for Game {
    fn into(self) -> GameDto {
        GameDto {
            id: self.id,
            summoner_id: self.myData.summoner.summoner_id,
            created_at: self.created_at.naive_utc(),
            champion_id: self.myData.champion_id,
            assists: self.myData.stats.assist,
            deaths: self.myData.stats.death,
            kills: self.myData.stats.kill,
            result: self.myData.stats.result,
            division: self.myData.tier_info.division,
            lp: self.myData.tier_info.lp,
            tier: self.myData.tier_info.tier,
            border_image_url: self.myData.tier_info.border_image_url,
            tier_image_url: self.myData.tier_info.tier_image_url,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MyData {
    champion_id: i64,
    stats: MyDataStats,
    tier_info: TierInfo,
    summoner: MyDataSummoner,
}

#[derive(Serialize, Deserialize)]
pub struct MyDataSummoner {
    summoner_id: String,
    name: String,
    level: i64,
    profile_image_url: String,
    acct_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct MyDataStats {
    assist: i64,
    death: i64,
    kill: i64,
    result: String,
}

#[derive(Serialize, Deserialize)]
pub struct TierInfo {
    division: Option<i64>,
    lp: Option<i64>,
    tier: Option<String>,
    border_image_url: Option<String>,
    tier_image_url: Option<String>,
}

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
            INSERT INTO game (
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
