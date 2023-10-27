use anyhow::{Context as Ctx, Result};
use serenity::model::prelude::component::InputText;
use serenity::model::prelude::{ChannelId, Guild, GuildId, Ready};
use serenity::prelude::{Context, EventHandler, GatewayIntents, TypeMapKey};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult, StandardFramework};
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
#[commands(delete_user, add_user, init)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("{} is connected!", _data_about_bot.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let facade;
        generate_facade_code!(ctx, facade);

        facade.start_workers(ctx.http.clone());
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        let facade;
        generate_facade_code!(ctx, facade);

        // copy guild name String
        let guild_name = guild.name.clone();

        match facade
            .init_guild_channel(guild.id.0 as i64, None, guild.name)
            .await
        {
            Ok(_) => {
                println!("Guild initialized: {} - {}", guild_name, guild.id.0);
            }
            Err(e) => {
                println!("Error initializing channel: {}", e);
            }
        }
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
#[description("Initialize the guild chat channel")]
async fn init(ctx: &Context, msg: &Message) -> CommandResult {
    let facade;
    generate_facade_code!(ctx, facade);

    let guild = msg.guild(&ctx.cache).context("No guild found")?;
    let channel_id = msg.channel_id.0;

    match facade
        .init_guild_channel(guild.id.0 as i64, Some(channel_id as i64), guild.name)
        .await
    {
        Ok(_) => {
            msg.reply(ctx, "Channel initialized!").await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("Error initializing channel: {}", e))
                .await?;
        }
    }

    Ok(())
}

#[command]
#[aliases("addUser")]
async fn add_user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let facade;
    generate_facade_code!(ctx, facade);

    let guild_id = msg.guild_id.context("No guild id found")?.0 as i64;

    match facade.add_user(args.rest(), guild_id).await {
        Ok(_) => {
            msg.reply(ctx, "User added!").await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("Error adding user: {}", e)).await?;
        }
    }

    Ok(())
}

#[command]
#[aliases("deleteUser")]
async fn delete_user(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let facade;
    generate_facade_code!(ctx, facade);

    match facade.delete_user(args.rest()).await {
        Ok(_) => {
            msg.reply(ctx, "User deleted!").await?;
        }
        Err(e) => {
            msg.reply(ctx, format!("Error deleting user: {}", e))
                .await?;
        }
    }

    Ok(())
}
