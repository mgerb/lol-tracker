use anyhow::Result;
use sqlx::{Pool, Sqlite};

#[derive(Debug, sqlx::FromRow)]
pub struct GuildDto {
    pub id: i64,
    pub chat_channel_id: Option<i64>,
    pub name: String,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl GuildDto {
    pub fn new(id: i64, chat_channel_id: Option<i64>, name: String) -> Self {
        Self {
            id,
            chat_channel_id,
            name,
            created_at: None,
            updated_at: None,
        }
    }

    pub async fn insert_or_ignore(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO guild (
               id,
               chat_channel_id,
               name
            )
            VALUES (?, ?, ?);
            "#,
            self.id,
            self.chat_channel_id,
            self.name
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE guild
            SET chat_channel_id = ?,
                name = ?
            WHERE id = ?;
            "#,
            self.chat_channel_id,
            self.name,
            self.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(pool: &Pool<Sqlite>) -> Result<Vec<GuildDto>> {
        let guilds = sqlx::query_as!(GuildDto, "SELECT * FROM guild")
            .fetch_all(pool)
            .await?;

        Ok(guilds)
    }
}
