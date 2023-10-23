use anyhow::{Context, Result};
use game::GameDto;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqlitePool, SqlitePoolOptions},
    QueryBuilder, Sqlite,
};

mod game;
mod summoner;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct SDto {
    id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // create db.sqlite file if not exists

    std::fs::File::create("db.sqlite").context("failed to create db.sqlite")?;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:db.sqlite")
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let summoner = summoner::Summoner::from_name("yeahistealdogs")
        .await
        .context("failed to get summoner")?;

    summoner.create(&pool).await?;

    let games = summoner.get_games().await?;

    for game in games {
        let game_dto: GameDto = game.into();
        game_dto.create(&pool).await?;
    }

    // let stream = sqlx::query_as::<_, GameDto>("SELECT * FROM game")
    //     .fetch(&pool)
    //     .await?;

    // let q: QueryBuilder<Sqlite> = QueryBuilder::new("");
    //
    // let d = sqlx::query_as!(GameDto, "SELECT * FROM game")
    //     .fetch_all(&pool)
    //     .await?;

    // print d

    //     let recs = sqlx::query(
    //         r#"
    // SELECT id, description, done
    // FROM todos
    // ORDER BY id
    //         "#,
    //     )
    //     .fetch_all(&pool)
    //     .await?;

    Ok(())
}
