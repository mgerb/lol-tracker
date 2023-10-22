use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> Result<()> {
    let summoner_id = get_summoner_id("plexs").await?;
    let stats = get_stats(&summoner_id).await?;

    Ok(())
}

async fn get_summoner_id(name: &str) -> Result<String> {
    let body = reqwest::get(format!("https://www.op.gg/_next/data/E6tX-RCMrF_ZUcw3Zom88/en_US/summoners/na/plexs.json?region=na&summoner={}", name))
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let summoner_id = json["pageProps"]["data"]["summoner_id"]
        .as_str()
        .context("name not found")?;

    Ok(summoner_id.to_string())
}

#[derive(Serialize, Deserialize)]
struct Game {
    id: String,
    myData: MyData,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
struct MyData {
    champion_id: u32,
    stats: MyDataStats,
    tier_info: TierInfo,
}

#[derive(Serialize, Deserialize)]
struct MyDataStats {
    assist: u32,
    death: u32,
    kill: u32,
    result: String,
}

#[derive(Serialize, Deserialize)]
struct TierInfo {
    division: Option<u32>,
    lp: Option<u32>,
    tier: Option<String>,
    border_image_url: Option<String>,
    tier_image_url: Option<String>,
}

async fn get_stats(summoner_id: &str) -> Result<Vec<Game>> {
    let body = reqwest::get(format!("https://op.gg/api/v1.0/internal/bypass/games/na/summoners/{}?&limit=5&hl=en_US&game_type=total", summoner_id))
        .await?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&body)?;

    let games = json["data"].as_array().context("games not found")?;

    let mut all_games: Vec<Game> = vec![];

    for g in games {
        let game: Game = serde_json::from_value(g.clone())?;

        {
            println!("{}", serde_json::to_string_pretty(&game)?);
        }

        all_games.push(game);
    }

    Ok(all_games)
}
