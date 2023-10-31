use crate::dtos::{game_dto::GameDto, summoner_dto::SummonerDto};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ApiStrategy
where
    Self: Send + Sync,
{
    async fn get_summoner(&self, summoner_name: &str, guild_id: i64) -> Result<SummonerDto>;
    async fn get_games(&self, summoner_id: &str) -> Result<Vec<GameDto>>;
}
