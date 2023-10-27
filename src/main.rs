use anyhow::Result;

mod bot;
mod db;
mod dtos;
mod facade;
mod op_gg_api;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");

    let facade = facade::Facade::new().await?;

    bot::start(facade).await?;

    Ok(())
}
