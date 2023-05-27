use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{migrate::MigrateDatabase, FromRow, Pool, Row, Sqlite, SqlitePool};

use super::DB_URL;

#[derive(Clone, FromRow, Debug)]
struct ChannelList {
    tag_id: i64,
    channel_id: String,
}

pub async fn create_tables() -> anyhow::Result<()> {
    let pool = SqlitePool::connect(DB_URL).await?;
    create_tables_with_pool(pool).await
}

pub async fn create_tables_with_pool(pool: Pool<Sqlite>) -> anyhow::Result<()> {
    let _dist = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS dist 
    (
        user_id TEXT NOT NULL,
        tag_id INTEGER NOT NULL, 
        dist_channel_id TEXT NOT NULL,
        PRIMARY KEY(user_id, tag_id, dist_channel_id),
        FOREIGN KEY (tag_id) REFERENCES user_folder(tag_id) ON DELETE CASCADE
    );",
    )
    .execute(&pool)
    .await?;
    let _user_folder = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS user_folder 
    (
        tag_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, 
        tag_name TEXT NOT NULL,
        owner_id TEXT NOT NULL,
        bot BOOLEAN NOT NULL DEFAULT false,
        UNIQUE (tag_name, owner_id)
    );",
    )
    .execute(&pool)
    .await?;
    let _ = sqlx::query!(
        "CREATE TABLE IF NOT EXISTS channel_list
    (
        tag_id INTEGER NOT NULL, 
        channel_id TEXT NOT NULL,
        PRIMARY KEY(tag_id, channel_id),
        FOREIGN KEY (tag_id) REFERENCES user_folder(tag_id) ON DELETE CASCADE
    );",
    )
    .execute(&pool)
    .await?;

    Ok(())
}

async fn insert_user_folder(tag: String, owner_id: SlackUserId, bot: bool) -> anyhow::Result<()> {
    let db = SqlitePool::connect(DB_URL).await?;

    let _query = sqlx::query("INSERT INTO user_folder (tag_name, owner_id) VALUES ($1, $2);")
        .bind(tag)
        .bind(owner_id.to_string())
        .execute(&db)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn table_list(pool: Pool<Sqlite>) -> Vec<String> {
        let result = sqlx::query(
            "SELECT name
             FROM sqlite_schema
             WHERE type ='table' 
             AND name NOT LIKE 'sqlite_%';",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get::<String, &str>("name"))
        .collect::<Vec<_>>();
        result
    }

    #[sqlx::test]
    async fn create_test(pool: Pool<Sqlite>) {
        create_tables_with_pool(pool.clone()).await.unwrap();
        let list = table_list(pool).await;

        let desired_tables = vec!["dist", "user_folder", "channel_list"]
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        let contain = desired_tables.iter().all(|s| list.contains(s));

        assert!(contain);
    }
}
