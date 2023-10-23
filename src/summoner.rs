use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

use crate::game::Game;

pub struct Summoner {
    id: String,
    name: String,
}

impl Summoner {
    pub async fn from_name(name: &str) -> Result<Self> {
        let body = reqwest::get(format!("https://www.op.gg/_next/data/E6tX-RCMrF_ZUcw3Zom88/en_US/summoners/na/plexs.json?region=na&summoner={}", name))
        .await?
        .text()
        .await?;

        let json: serde_json::Value = serde_json::from_str(&body)?;

        let summoner_id = json["pageProps"]["data"]["summoner_id"]
            .as_str()
            .context("name not found")?;

        Ok(Self {
            id: summoner_id.to_string(),
            name: name.to_string(),
        })
    }

    pub async fn get_games(&self) -> Result<Vec<Game>> {
        let body = reqwest::get(format!("https://op.gg/api/v1.0/internal/bypass/games/na/summoners/{}?&limit=5&hl=en_US&game_type=total", self.id))
        .await?
        .text()
        .await?;

        let json: serde_json::Value = serde_json::from_str(&body)?;

        let games = json["data"].as_array().context("games not found")?;

        let mut all_games: Vec<Game> = vec![];

        for g in games {
            let game: Game = serde_json::from_value(g.clone())?;
            all_games.push(game);
        }

        Ok(all_games)
    }

    pub async fn create(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO summoner (
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
}
