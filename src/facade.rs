use anyhow::Result;
use sqlx::{Pool, Sqlite};
use tokio::task::JoinSet;

use crate::{db, dtos::summoner_dto::SummonerDto, op_gg_api};

/// Facade to interact with database and op.gg api
pub struct Facade {
    pool: Pool<Sqlite>,
}

impl Facade {
    /// Create a new Facade
    pub async fn new() -> Result<Self> {
        let pool = db::create_db().await?;
        Ok(Self { pool })
    }

    pub async fn add_user(&self, summoner_name: &str) -> Result<()> {
        let summoner = op_gg_api::get_summoner(summoner_name).await?.to_dto();

        summoner.create(&self.pool).await?;
        Ok(())
    }

    pub async fn start_workers(&mut self) -> Result<()> {
        let mut set = JoinSet::new();

        let pool = self.pool.clone();

        set.spawn(async move { Self::summoner_worker(pool).await });

        while let Some(res) = set.join_next().await {
            let _ = res?;
        }

        Ok(())
    }

    async fn summoner_worker(pool: Pool<Sqlite>) -> Result<()> {
        loop {
            let summoners = SummonerDto::get_all(&pool).await?;

            for s in summoners {
                let games = op_gg_api::get_games(s.id.as_str()).await?;
                for game in games {
                    game.to_dto().create(&pool).await?;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}
