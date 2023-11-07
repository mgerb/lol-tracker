use std::sync::Arc;

use anyhow::Result;

mod api_strategy;
mod bot;
mod db;
mod dtos;
mod facade;
mod league_of_graphs_api;
mod op_gg_api;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    // Replace with your own strategy if necessary
    let strategy = league_of_graphs_api::LeagueOfGraphsApiStrategy;

    let facade = facade::Facade::new(Arc::new(strategy)).await?;

    facade.startup_tasks().await?;

    bot::start(facade).await?;

    Ok(())
}
