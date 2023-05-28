use anyhow::Context;
use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};

use super::{utils, DB_URL};

#[derive(Clone, FromRow, Debug)]
struct UserFolder {
    tag_id: i64,
    owner_id: String,
    tag_name: String,
    bot: bool,
}

// register channel to tag
pub async fn register_channel(
    tag_name: &str,
    channel: SlackChannelId,
    user: SlackUserId,
) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(DB_URL).await?;
    register_channel_with_pool(tag_name, channel, user, pool).await
}
pub async fn register_channel_with_pool(
    tag_name: &str,
    channel: SlackChannelId,
    user: SlackUserId,
    pool: Pool<Sqlite>,
) -> anyhow::Result<()> {
    let owner_id = user.to_string();
    let _query_uf = sqlx::query!(
        "INSERT INTO user_folder (tag_name, owner_id) VALUES ($1, $2);",
        tag_name,
        owner_id
    )
    .execute(&pool)
    .await?;

    let tag_id = utils::fetch_tag_id_with_pool(user, tag_name, &pool).await?;

    let channel_id = channel.to_string();
    let _query_cl = sqlx::query!(
        "INSERT INTO channel_list (channel_id, tag_id) VALUES ($1,$2);",
        channel_id,
        tag_id
    )
    .execute(&pool)
    .await?;

    Ok(())
}
pub async fn unregister_channel(
    tag_name: &str,
    channel: SlackChannelId,
    user: SlackUserId,
) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(DB_URL).await?;
    unregister_channel_with_url(tag_name, user, pool).await
}

// Unregistration of channel_list table is automatic due to cascade constraints
pub async fn unregister_channel_with_url(
    tag_name: &str,
    user: SlackUserId,
    pool: Pool<Sqlite>,
) -> anyhow::Result<()> {
    let owner_id = user.to_string();
    let _query_uf = sqlx::query!(
        "DELETE FROM user_folder WHERE tag_name = $1 AND owner_id = $2;",
        tag_name,
        owner_id
    )
    .execute(&pool)
    .await?;
    Ok(())
}
pub async fn retrieve_bot(
    tag_name: &str,
    user: SlackUserId,
    retrieve_bot: bool,
) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(DB_URL).await?;
    retrieve_bot_with_pool(tag_name, user, retrieve_bot, pool).await
}
pub async fn retrieve_bot_with_pool(
    tag_name: &str,
    user: SlackUserId,
    retrieve_bot: bool,
    pool: Pool<Sqlite>,
) -> anyhow::Result<()> {
    let owner_id = user.to_string();
    let _query_uf = sqlx::query!(
        "UPDATE user_folder SET bot = $1 WHERE tag_name = $2 AND owner_id = $3;",
        retrieve_bot,
        tag_name,
        owner_id
    )
    .execute(&pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use slack_morphism::SlackUserId;
    use sqlx::{Row, SqlitePool};

    use super::super::DB_TEST_URL;
    use super::*;

    async fn register_test(
        pool: Pool<Sqlite>,
    ) -> anyhow::Result<(String, SlackChannelId, SlackUserId)> {
        let tag_name = "test";
        let channel = SlackChannelId::new("C01234".to_string());
        let user = SlackUserId::new("U0987".to_string());
        register_channel_with_pool(tag_name, channel.clone(), user.clone(), pool).await?;
        Ok((tag_name.to_string(), channel, user))
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_register_channel(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (_, channel, _) = register_test(pool.clone()).await?;

        let result_channel_id = sqlx::query!(
            "SELECT uf.tag_id, cl.channel_id
             FROM user_folder AS uf INNER JOIN channel_list AS cl
             ON cl.tag_id = uf.tag_id
             ",
        )
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|r| r.channel_id.clone())
        .collect::<Vec<_>>();

        let target = channel.to_string();
        let is_registered = result_channel_id.contains(&target);
        assert!(is_registered);
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_unregister_channel(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag_name, channel, user) = register_test(pool.clone()).await?;

        unregister_channel_with_url(&tag_name, user, pool.clone()).await?;

        let result_channel_id = sqlx::query!(
            "SELECT channel_id
            FROM channel_list
            WHERE channel_id = 'C01234'
            ",
        )
        .fetch_one(&pool)
        .await;

        let exist_channel_id = result_channel_id.is_err();
        assert!(exist_channel_id);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_retrieve_bot(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag_name, channel, user) = register_test(pool.clone()).await?;

        let result_bot = sqlx::query!(
            "SELECT bot
            FROM user_folder
            WHERE tag_name = 'test' AND owner_id = 'U0987'
            ",
        )
        .fetch_one(&pool)
        .await?
        .bot;

        assert!(!result_bot);

        retrieve_bot_with_pool(&tag_name, user, true, pool.clone()).await?;

        let result_bot = sqlx::query!(
            "SELECT bot
            FROM user_folder
            WHERE tag_name = 'test' AND owner_id = 'U0987'
            ",
        )
        .fetch_one(&pool)
        .await?
        .bot;

        assert!(result_bot);

        Ok(())
    }
}
