use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn _create_tables() -> anyhow::Result<()> {
    let db_url = super::db_url()?;
    let pool = SqlitePool::connect(&db_url).await?;
    _create_tables_with_pool(pool).await
}

pub async fn _create_tables_with_pool(pool: Pool<Sqlite>) -> anyhow::Result<()> {
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
        valid_count INTEGER NOT NULL DEFAULT 0,

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

#[cfg(test)]
mod tests {
    use sqlx::Row;

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
        _create_tables_with_pool(pool.clone()).await.unwrap();
        let list = table_list(pool).await;

        let desired_tables = vec!["dist", "user_folder", "channel_list"]
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        let contain = desired_tables.iter().all(|s| list.contains(s));

        assert!(contain);
    }
}
