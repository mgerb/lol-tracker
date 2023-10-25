use anyhow::{Context as Ctx, Result};
use serenity::model::prelude::{ChannelId, GuildId, Ready};
use serenity::prelude::{Context, EventHandler, GatewayIntents, TypeMapKey};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::{async_trait, Client};

use crate::facade::Facade;

// TODO: clean this shitty macro up
macro_rules! generate_facade_code {
    ($ctx:ident, $facade:ident) => {
        $facade = {
            let data_read = $ctx.data.read().await;
            data_read.get::<FacadeContainer>().unwrap().clone()
        };

        let mut $facade = $facade.write().await;
    };
}

struct FacadeContainer;

impl TypeMapKey for FacadeContainer {
    type Value = Arc<RwLock<Facade>>;
}

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("{} is connected!", _data_about_bot.user.name);
    }

    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {
        let facade;
        generate_facade_code!(_ctx, facade);

        facade.start_workers(_ctx.http.clone());
    }
}

pub async fn start(facade: Facade) -> Result<()> {
    let prefix = env::var("BOT_PREFIX").context("unable to parse BOT_PREFIX from env file")?;
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").context("unable to parse DISCORD_TOKEN from env file")?;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .context("Error creating client")?;

    // Set the facade in the client's data
    {
        let mut data = client.data.write().await;
        data.insert::<FacadeContainer>(Arc::new(RwLock::new(facade)));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        return Err(anyhow::anyhow!("Error starting client: {:?}", why));
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
