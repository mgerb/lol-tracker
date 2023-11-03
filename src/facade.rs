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
use url::Url;

use crate::{
    api_strategy::{self, ApiStrategy},
    db,
    dtos::{
        active_game_dto::ActiveGameDto, game_dto::GameDto, guild_dto::GuildDto, log_dto::LogDto,
        summoner_dto::SummonerDto,
    },
    op_gg_api,
};

static GAME_WATCHER_INTERVAL: u64 = 60;
static SUMMONER_API_INTERVAL: u64 = 180;
static ACTIVE_GAME_INTERVAL: u64 = 60;

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

    pub async fn log_info(&self, message: &str) {
        LogDto::info(&self.pool, message).await;
    }

    pub async fn log_error(&self, message: &str) {
        LogDto::error(&self.pool, message).await;
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
            match self.api_strategy.get_games(s.id.as_str()).await {
                Ok(games) => {
                    // Fetch and store new games
                    for mut game in games {
                        game.notified = true;
                        game.upsert(&self.pool).await?;
                    }
                }
                Err(e) => {
                    println!("startup_tasks: {}", e.to_string().as_str());
                }
            }
            // Set all games to "notified"
            GameDto::set_all_notified(&self.pool).await?;
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
        let pool = self.pool.clone();
        let api_strategy = self.api_strategy.clone();
        self.join_set
            .spawn(async move { Self::start_summoner_api_worker(api_strategy, pool).await });

        let pool = self.pool.clone();
        let http_clone = http.clone();
        self.join_set
            .spawn(async move { Self::start_game_watcher_worker(pool, http_clone).await });

        let pool = self.pool.clone();
        let api_strategy = self.api_strategy.clone();
        self.join_set
            .spawn(async move { Self::start_active_game_api_worker(api_strategy, pool).await });

        let pool = self.pool.clone();
        let http_clone = http.clone();
        self.join_set
            .spawn(async move { Self::start_active_game_watcher_worker(pool, http_clone).await });
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
                    .await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(GAME_WATCHER_INTERVAL)).await;
        }
    }

    async fn game_watcher_worker(pool: &Pool<Sqlite>, http: &Http) -> Result<()> {
        let summoners = SummonerDto::get_all(&pool).await?;

        for s in summoners {
            let games = GameDto::get_unnotified_games_for_summoner(&pool, &s.id).await?;
            let guild = s.get_guild(&pool).await?;
            let tier = s.tier.unwrap_or("".to_string());
            let division = s.division.unwrap_or("".to_string());
            let lp = s.lp.unwrap_or(0);

            for mut game in games {
                if let Some(chat_channel_id) = guild.chat_channel_id {
                    let mut embed = CreateEmbed::default();
                    let color = if game.win {
                        // Green #15e55a
                        Colour::new(0x15e55a)
                    } else {
                        // Red #e55a5a
                        Colour::new(0xe55a5a)
                    };

                    let thumbnail_url = format!(
                        "https://ddragon.leagueoflegends.com/cdn/13.21.1/img/champion/{}.png",
                        game.champion_name
                    );
                    let thumbnail_url = Url::parse(&thumbnail_url)?.to_string();

                    let lp_change = game
                        .lp_change
                        .map(|lp| {
                            if lp > 0 {
                                format!("+{}", lp)
                            } else {
                                lp.to_string()
                            }
                        })
                        .map_or("".to_string(), |lp| format!("{} league point(s)!", lp));

                    let match_url = format!("https://leagueofgraphs.com{}", game.id);
                    let match_url = Url::parse(&match_url)?.to_string();
                    let icon_url = Url::parse("https://cdn.discordapp.com/icons/101198129352691712/362b75dce5e0960e1c35250928e268f2.webp?size=128")?.to_string();
                    let title = if game.win { "Victory" } else { "Defeat" };

                    embed
                        .title(title.to_string())
                        .url(match_url)
                        .description(format!(
                            "{} {} ({} {} {} lp)",
                            s.name, lp_change, tier, division, lp,
                        ))
                        .color(color)
                        .timestamp(Timestamp::from_unix_timestamp(game.game_created_at)?)
                        .author(|a| a.name("Game Results").icon_url(icon_url))
                        .fields(vec![
                            (
                                "Score",
                                format!("{}/{}/{}", game.kills, game.deaths, game.assists),
                                true,
                            ),
                            ("Champion", game.champion_name.clone(), true),
                            ("Queue", game.game_mode.clone(), true),
                        ])
                        .thumbnail(thumbnail_url);

                    ChannelId(chat_channel_id as u64)
                        .send_message(http, |m| m.set_embed(embed))
                        .await?;
                } else {
                    LogDto::error(
                        &pool,
                        &format!("No chat channel set for guild: {}", guild.id),
                    )
                    .await;
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
                        &format!("start_summoner_api_worker: {}", &e.to_string()),
                    )
                    .await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(SUMMONER_API_INTERVAL)).await;
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

    async fn start_active_game_api_worker(
        api_strategy: Arc<dyn ApiStrategy>,
        pool: Pool<Sqlite>,
    ) -> Result<()> {
        loop {
            match Self::active_game_api_worker(api_strategy.clone(), &pool).await {
                Ok(_) => {}
                Err(e) => {
                    LogDto::error(
                        &pool,
                        &format!("start_active_game_worker: {}", &e.to_string()),
                    )
                    .await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(ACTIVE_GAME_INTERVAL)).await;
        }
    }

    async fn active_game_api_worker(
        api_strategy: Arc<dyn ApiStrategy>,
        pool: &Pool<Sqlite>,
    ) -> Result<()> {
        let summoners = SummonerDto::get_all(&pool).await?;

        for s in summoners {
            if let Some(active_game) = api_strategy
                .get_active_game(s.id.as_str(), s.name.as_str())
                .await?
            {
                active_game.insert_or_ignore(&pool).await?;
            }
        }

        Ok(())
    }

    async fn start_active_game_watcher_worker(pool: Pool<Sqlite>, http: Arc<Http>) -> Result<()> {
        loop {
            match Self::active_game_watcher_worker(&pool, &http).await {
                Ok(_) => {}
                Err(e) => {
                    LogDto::error(
                        &pool,
                        &format!("start_active_game_watcher_worker: {}", &e.to_string()),
                    )
                    .await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(ACTIVE_GAME_INTERVAL)).await;
        }
    }

    async fn active_game_watcher_worker(pool: &Pool<Sqlite>, http: &Http) -> Result<()> {
        let active_games = ActiveGameDto::get_unnotified_active_games(&pool).await?;

        for mut active_game in active_games {
            let summoner = SummonerDto::get(&pool, active_game.summoner_id.as_str()).await?;
            let guild = summoner.get_guild(&pool).await?;

            if let Some(chat_channel_id) = guild.chat_channel_id {
                let mut embed = CreateEmbed::default();
                // Yello #e5e55a
                let color = Colour::new(0xe5e55a);

                let thumbnail_url = format!(
                    "https://ddragon.leagueoflegends.com/cdn/13.21.1/img/champion/{}.png",
                    active_game.champion
                );
                let thumbnail_url = Url::parse(&thumbnail_url)?.to_string();

                let match_url = format!("https://porofessor.gg/live/na/{}", summoner.name);
                let match_url = Url::parse(&match_url)?.to_string();
                let icon_url = Url::parse("https://cdn.discordapp.com/icons/101198129352691712/362b75dce5e0960e1c35250928e268f2.webp?size=128")?.to_string();

                embed
                    .title(format!("In game {}", active_game.game_mode))
                    .url(match_url)
                    .description(format!(
                        "{} is currently in a game with {}",
                        summoner.name, active_game.champion
                    ))
                    .color(color)
                    .timestamp(Timestamp::from_unix_timestamp(active_game.game_created_at)?)
                    .author(|a| a.name("Game status").icon_url(icon_url))
                    .fields(vec![
                        ("Champion", active_game.champion.clone(), true),
                        ("Role", active_game.role.clone(), true),
                    ])
                    .thumbnail(thumbnail_url);

                ChannelId(chat_channel_id as u64)
                    .send_message(http, |m| m.set_embed(embed))
                    .await?;

                active_game.notified = true;
                active_game.upsert(&pool).await?;
            } else {
                LogDto::error(
                    &pool,
                    &format!("No chat channel set for guild: {}", guild.id),
                )
                .await;
            }
        }

        Ok(())
    }
}
