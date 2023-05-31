use sqlx::{migrate::MigrateDatabase, Sqlite};

pub const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if !Sqlite::database_exists(DB_URL).await? {
        Sqlite::create_database(DB_URL);
        create_table::_create_tables().await?;
    }

    Ok(())
}

mod create_table {
    use sqlx::{Pool, Sqlite, SqlitePool};

    use crate::DB_URL;

    pub async fn _create_tables() -> anyhow::Result<()> {
        let pool = SqlitePool::connect(DB_URL).await?;
        _create_tables_with_pool(pool).await
    }

    pub async fn _create_tables_with_pool(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let _dist = sqlx::query(
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
        let _user_folder = sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_folder 
    (
        tag_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, 
        tag_name TEXT NOT NULL,
        owner_id TEXT NOT NULL,
        bot BOOLEAN NOT NULL DEFAULT false,
        valid_count INTEGER NOT NULL DEFAULT 0,

        UNIQUE (tag_name, owner_id)
    );",
        )
        .execute(&pool)
        .await?;
        let _ = sqlx::query(
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
}
