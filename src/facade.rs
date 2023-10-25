use std::sync::Arc;

use anyhow::{Context, Result};
use serenity::{http::Http, model::prelude::ChannelId};
use sqlx::{Pool, Sqlite};
use tokio::task::JoinSet;

use crate::{db, dtos::summoner_dto::SummonerDto, op_gg_api};

/// Facade to interact with database and op.gg api
pub struct Facade {
    pool: Pool<Sqlite>,
    join_set: JoinSet<Result<()>>,
}

impl Facade {
    /// Create a new Facade
    pub async fn new() -> Result<Self> {
        let pool = db::create_db().await?;
        Ok(Self {
            pool,
            join_set: JoinSet::new(),
        })
    }

    pub async fn add_user(&self, summoner_name: &str) -> Result<()> {
        let summoner = op_gg_api::get_summoner(summoner_name).await?.to_dto();

        summoner.create(&self.pool).await?;
        Ok(())
    }

    pub fn start_workers(&mut self, http: Arc<Http>) {
        let pool = self.pool.clone();

        self.join_set
            .spawn(async move { Self::summoner_worker(pool, http).await });
    }

    async fn summoner_worker(pool: Pool<Sqlite>, http: Arc<Http>) -> Result<()> {
        loop {
            let summoners = SummonerDto::get_all(&pool).await?;

            for s in summoners {
                let games = op_gg_api::get_games(s.id.as_str()).await?;
                for game in games {
                    game.to_dto().create(&pool).await?;
                }
            }

            ChannelId(401802817716748288)
                .say(&http, "Worker 1 finished. Sleeping for 60 seconds")
                .await
                .context("summoner_worker: failed to send message")?;
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}
