use slack_morphism::SlackUserId;
use sqlx::{Pool, Sqlite, SqlitePool};

use super::DB_URL;

pub async fn fetch_tag_id_with_pool(
    owner_id: SlackUserId,
    tag_name: &str,
    pool: Pool<Sqlite>,
) -> anyhow::Result<i64> {
    let owner_id_str = owner_id.to_string();

    let tag_id = sqlx::query!(
        "
    SELECT tag_id
    FROM user_folder
    WHERE owner_id = $1 AND tag_name = $2 
    ",
        owner_id_str,
        tag_name
    )
    .fetch_one(&pool)
    .await?
    .tag_id;

    Ok(tag_id)
}

#[cfg(test)]
mod tests {
    use slack_morphism::SlackChannelId;
    use sqlx::Pool;

    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn fetch_tag_id_test(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let tag_name = "test";
        let owner = "U0987654".to_string();
        let owner_id = SlackUserId::new(owner.clone());

        let _reg_query = sqlx::query!(
            "
        INSERT INTO user_folder  (tag_name, owner_id) VALUES ($1, $2)
        ",
            tag_name,
            owner
        )
        .execute(&pool)
        .await?;

        let tag_id = fetch_tag_id_with_pool(owner_id, tag_name, pool.clone()).await?;

        let tag_id_fetch = sqlx::query!(
            "
        SELECT tag_id
        FROM user_folder
        WHERE tag_name = 'test' AND owner_id = 'U0987654'
        "
        )
        .fetch_one(&pool)
        .await?
        .tag_id;

        assert_eq!(tag_id_fetch, tag_id);

        Ok(())
    }
}
