use anyhow::Result;

mod db;
mod dtos;
mod facade;
mod op_gg_api;

#[tokio::main]
async fn main() -> Result<()> {
    let mut facade = facade::Facade::new().await?;

    facade.add_user("YeahIStealDogs").await?;

    facade.start_workers().await?;

    Ok(())
}
