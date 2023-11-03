use anyhow::Result;
use sqlx::{Pool, Sqlite};

#[derive(Debug, sqlx::FromRow)]
pub struct GameDto {
    pub id: String,
    pub summoner_id: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub game_created_at: i64,
    pub assists: i64,
    pub deaths: i64,
    pub kills: i64,
    pub win: bool,
    pub notified: bool,
    pub champion_name: String,
    pub game_mode: String,
    pub lp_change: Option<i64>,
    pub promotion_text: Option<String>,
}

impl GameDto {
    pub async fn insert_or_ignore(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO game (
                id,
                summoner_id,
                game_created_at,
                assists,
                deaths,
                kills,
                win,
                notified,
                lp_change,
                champion_name,
                game_mode,
                promotion_text
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.assists,
            self.deaths,
            self.kills,
            self.win,
            self.notified,
            self.lp_change,
            self.champion_name,
            self.game_mode,
            self.promotion_text
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO game (
                id,
                summoner_id,
                game_created_at,
                assists,
                deaths,
                kills,
                win,
                notified,
                lp_change,
                champion_name,
                game_mode,
                promotion_text
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.assists,
            self.deaths,
            self.kills,
            self.win,
            self.notified,
            self.lp_change,
            self.champion_name,
            self.game_mode,
            self.promotion_text
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<GameDto>> {
        let games = sqlx::query_as!(GameDto, "SELECT * FROM game")
            .fetch_all(pool)
            .await?;

        Ok(games)
    }

    pub async fn get_unnotified_games_for_summoner(
        pool: &Pool<Sqlite>,
        summoner_id: &str,
    ) -> Result<Vec<GameDto>> {
        let games = sqlx::query_as!(
            GameDto,
            r#"
            SELECT * FROM game
            WHERE summoner_id = ? AND notified = 0;
            "#,
            summoner_id
        )
        .fetch_all(pool)
        .await?;

        Ok(games)
    }

    pub async fn set_all_notified(pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE game
            SET notified = 1;
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
