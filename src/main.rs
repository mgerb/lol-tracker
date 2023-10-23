use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use dtos::summoner_dto::SummonerDto;
use game::GameDto;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqlitePool, SqlitePoolOptions},
    Pool, QueryBuilder, Sqlite,
};

mod db;
mod dtos;
mod game;
mod op_gg_api;
mod summoner;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct SDto {
    id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = db::create_db().await?;

    let summoner = op_gg_api::get_summoner("YeahIStealDogs").await?.to_dto();

    summoner.create(&pool).await?;

    let games = op_gg_api::get_games(summoner.id.as_str()).await?;

    for game in games {
        let game_dto = game.to_dto();
        game_dto.create(&pool).await?;
    }

    tokio::spawn(async move { start_worker(pool.clone()).await });

    Ok(())
}

async fn start_worker(pool: Pool<Sqlite>) -> Result<()> {
    let summoners = SummonerDto::get_all(&pool).await?;

    for s in summoners {
        let games = op_gg_api::get_games(s.id.as_str()).await?;
        for game in games {
            game.to_dto().create(&pool).await?;
        }
    }

    Ok(())
}
