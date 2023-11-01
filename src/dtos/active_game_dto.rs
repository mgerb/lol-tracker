use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

#[derive(Debug, sqlx::FromRow)]
pub struct ActiveGameDto {
    pub id: String,
    pub summoner_id: String,
    pub created_at: Option<i64>,
    pub game_created_at: i64,
    pub champion: String,
    pub role: String,
    pub spectate_link: String,
    pub notified: bool,
    pub game_mode: String,
}

impl ActiveGameDto {
    pub async fn upsert(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO active_game (id, summoner_id, game_created_at, created_at, champion, role, spectate_link, notified, game_mode)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.created_at,
            self.champion,
            self.role,
            self.spectate_link,
            self.notified,
            self.game_mode,
        )
        .execute(pool)
        .await
        .context("Failed to upsert active game")?;

        Ok(())
    }

    pub async fn insert_or_ignore(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO active_game (id, summoner_id, game_created_at, created_at, champion, role, spectate_link, notified, game_mode)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.id,
            self.summoner_id,
            self.game_created_at,
            self.created_at,
            self.champion,
            self.role,
            self.spectate_link,
            self.notified,
            self.game_mode,
        )
        .execute(pool)
        .await
        .context("Failed to insert active game")?;

        Ok(())
    }

    pub async fn get_unnotified_active_games(pool: &Pool<Sqlite>) -> Result<Vec<ActiveGameDto>> {
        let active_games = sqlx::query_as!(
            ActiveGameDto,
            r#"
            SELECT *
            FROM active_game
            WHERE notified = 0
            "#,
        )
        .fetch_all(pool)
        .await
        .context("Failed to get unnotified active games")?;

        Ok(active_games)
    }
}
