use std::sync::Arc;

use anyhow::Result;

mod api_strategy;
mod bot;
mod db;
mod dtos;
mod facade;
mod log_api;
mod op_gg_api;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");

    let facade = facade::Facade::new(Arc::new(log_api::LogApiStrategy)).await?;

    facade.startup_tasks().await?;

    bot::start(facade).await?;

    Ok(())
}
