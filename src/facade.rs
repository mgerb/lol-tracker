use std::sync::Arc;

use anyhow::{Context, Result};
use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{prelude::ChannelId, Timestamp},
    utils::Colour,
};
use sqlx::{Pool, Sqlite};
use tokio::task::JoinSet;

use crate::{
    api_strategy::{self, ApiStrategy},
    db,
    dtos::{game_dto::GameDto, guild_dto::GuildDto, log_dto::LogDto, summoner_dto::SummonerDto},
    op_gg_api,
};

/// Facade to interact with database and op.gg api
pub struct Facade {
    pool: Pool<Sqlite>,
    join_set: JoinSet<Result<()>>,
    api_strategy: Arc<dyn ApiStrategy>,
}

impl Facade {
    /// Create a new Facade
    pub async fn new(api_strategy: Arc<dyn ApiStrategy>) -> Result<Self> {
        let pool = db::create_db().await?;
        Ok(Self {
            pool,
            join_set: JoinSet::new(),
            api_strategy,
        })
    }

    pub async fn log_info(&self, message: &str) -> Result<()> {
        LogDto::info(&self.pool, message).await?;
        Ok(())
    }

    pub async fn log_error(&self, message: &str) -> Result<()> {
        LogDto::error(&self.pool, message).await?;
        Ok(())
    }

    pub async fn init_guild(
        &self,
        guild_id: i64,
        chat_channel_id: Option<i64>,
        name: String,
    ) -> Result<()> {
        GuildDto::new(guild_id, chat_channel_id, name)
            .insert_or_ignore(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_guild_channel(
        &self,
        guild_id: i64,
        chat_channel_id: Option<i64>,
        name: String,
    ) -> Result<()> {
        GuildDto::new(guild_id, chat_channel_id, name)
            .update(&self.pool)
            .await?;
        Ok(())
    }

    /// - refresh games for all users
    /// - set all games to notified
    ///
    /// This makes it so that we don't spam notifications from stale games.
    pub async fn startup_tasks(&self) -> Result<()> {
        let summoners = SummonerDto::get_all(&self.pool).await?;

        for s in summoners {
            let games = self.api_strategy.get_games(s.id.as_str()).await?;
            for mut game in games {
                game.notified = true;
                game.upsert(&self.pool).await?;
            }
        }

        Ok(())
    }

    /// - fetch user from api
    /// - insert user into database
    /// - fetch all games for user
    /// - set all games to notified
    /// - insert games into database
    pub async fn add_user(&self, summoner_name: &str, guild_id: i64) -> Result<()> {
        let summoner = self
            .api_strategy
            .get_summoner(summoner_name, guild_id)
            .await?;

        summoner.insert_or_ignore(&self.pool).await?;

        // Fetch all games for the user and set to notified
        let games = self.api_strategy.get_games(summoner.id.as_str()).await?;
        for mut game in games {
            game.notified = true;
            game.upsert(&self.pool).await?;
        }

        Ok(())
    }

    /// - delete user from database
    pub async fn delete_user(&self, summoner_name: &str) -> Result<()> {
        SummonerDto::delete(&self.pool, summoner_name).await?;
        Ok(())
    }

    /// - start all workers
    pub fn start_workers(&mut self, http: Arc<Http>) {
        let pool1 = self.pool.clone();
        let pool2 = self.pool.clone();
        let api_strategy = self.api_strategy.clone();

        self.join_set
            .spawn(async move { Self::start_summoner_api_worker(api_strategy, pool1).await });
        self.join_set
            .spawn(async move { Self::start_game_watcher_worker(pool2, http).await });
    }

    async fn start_game_watcher_worker(pool: Pool<Sqlite>, http: Arc<Http>) -> Result<()> {
        loop {
            match Self::game_watcher_worker(&pool, &http).await {
                Ok(_) => {}
                Err(e) => {
                    LogDto::error(
                        &pool,
                        format!("start_game_watcher_worker: {}", e.to_string().as_str()).as_str(),
                    )
                    .await?;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(180)).await;
        }
    }

    async fn game_watcher_worker(pool: &Pool<Sqlite>, http: &Http) -> Result<()> {
        let summoners = SummonerDto::get_all(&pool).await?;

        for s in summoners {
            let games = s.get_unnotified_games(&pool).await?;
            let guild = s.get_guild(&pool).await?;
            let tier = s.tier.unwrap_or("".to_string());
            let division = s.division.unwrap_or("".to_string());
            let lp = s.lp.unwrap_or(0);

            for mut game in games {
                if let Some(chat_channel_id) = guild.chat_channel_id {
                    let mut embed = CreateEmbed::default();
                    let color = if game.win {
                        Colour::new(0x15e55a)
                    } else {
                        Colour::new(0xe55a5a)
                    };

                    embed
                        .title("Victory")
                        .url("TODO:")
                        .description(format!(
                            "YeahIStealDogs - {} league point(s)! ({} {} {} lp)",
                            game.lp_change.unwrap_or(0),
                            tier,
                            division,
                            lp
                        ))
                        .color(color)
                        .timestamp(Timestamp::from_unix_timestamp(game.game_created_at)?)
                        .author(|a| {
                            a.name("Game Results")
                            // .icon_url(format!("http://ddragon.leagueoflegends.com/cdn/13.21.1/img/champion/{}.png", game.champion_name))
                        })
                        .fields(vec![
                            (
                                "Score",
                                format!("{}/{}/{}", game.kills, game.deaths, game.assists),
                                true,
                            ),
                            ("Champion", game.champion_name.clone(), true),
                            ("Queue", game.game_mode.clone(), true),
                        ])
                        .thumbnail(format!(
                            "http://ddragon.leagueoflegends.com/cdn/13.21.1/img/champion/{}.png",
                            game.champion_name
                        ));

                    ChannelId(chat_channel_id as u64)
                        .send_message(&http, |m| m.set_embed(embed))
                        .await
                        .context(format!(
                            "Unable to notify channel {} for guild {}",
                            chat_channel_id, guild.id
                        ))?;
                } else {
                    LogDto::error(
                        &pool,
                        format!("No chat channel set for guild: {}", guild.id).as_str(),
                    )
                    .await?;
                }

                game.notified = true;
                game.upsert(&pool).await?;
            }
        }

        Ok(())
    }

    /// - start summoner worker to run every minute
    async fn start_summoner_api_worker(
        api_strategy: Arc<dyn ApiStrategy>,
        pool: Pool<Sqlite>,
    ) -> Result<()> {
        loop {
            match Self::summoner_api_worker(api_strategy.clone(), &pool).await {
                Ok(_) => {}
                Err(e) => {
                    LogDto::error(
                        &pool,
                        format!("start_summoner_api_worker: {}", e.to_string().as_str()).as_str(),
                    )
                    .await?;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// - fetch all summoners from database
    /// - fetch all games for each summoner
    /// - insert or ignore games into database
    async fn summoner_api_worker(
        api_strategy: Arc<dyn ApiStrategy>,
        pool: &Pool<Sqlite>,
    ) -> Result<()> {
        let summoners = SummonerDto::get_all(&pool).await?;

        for s in summoners {
            // Fetch summoner and update stats
            api_strategy
                .get_summoner(s.name.as_str(), s.guild_id)
                .await?
                .upsert(&pool)
                .await?;

            let games = api_strategy.get_games(s.id.as_str()).await?;
            for game in games {
                game.insert_or_ignore(&pool).await?;
            }
        }

        Ok(())
    }
}
