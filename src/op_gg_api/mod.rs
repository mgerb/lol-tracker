use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::dtos::{game_dto::GameDto, summoner_dto::SummonerDto};

pub struct SummonerResponse {
    pub id: String,
    pub name: String,
}

impl SummonerResponse {
    pub fn to_dto(self) -> SummonerDto {
        SummonerDto {
            id: self.id,
            name: self.name,
            created_at: None,
        }
    }
}

pub async fn get_summoner(name: &str) -> Result<SummonerResponse> {
    let body = reqwest::get(format!("https://www.op.gg/_next/data/E6tX-RCMrF_ZUcw3Zom88/en_US/summoners/na/{}.json?region=na&summoner={}", name, name))
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let summoner_id = json["pageProps"]["data"]["summoner_id"]
        .as_str()
        .context("name not found")?;

    Ok(SummonerResponse {
        id: summoner_id.to_string(),
        name: name.to_string(),
    })
}

#[derive(Serialize, Deserialize)]
pub struct GameResponse {
    id: String,
    myData: MyData,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl GameResponse {
    pub fn to_dto(self) -> GameDto {
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

pub async fn get_games(summoner_id: &str) -> Result<Vec<GameResponse>> {
    let body = reqwest::get(format!("https://op.gg/api/v1.0/internal/bypass/games/na/summoners/{}?&limit=5&hl=en_US&game_type=total", summoner_id))
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let games = json["data"].as_array().context("games not found")?;

    let mut all_games: Vec<GameResponse> = vec![];

    for g in games {
        let game: GameResponse = serde_json::from_value(g.clone())?;
        all_games.push(game);
    }

    Ok(all_games)
}
