use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{Pool, Sqlite, SqlitePool};

use super::utils;

// register channel to tag
pub async fn register_channel(
    tag_name: &str,
    channel: SlackChannelId,
    user: SlackUserId,
) -> anyhow::Result<()> {
    let db_url = super::db_url()?;
    let pool = SqlitePool::connect(&db_url).await?;
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
        "
        INSERT INTO user_folder (tag_name, owner_id) VALUES ($1, $2)
        ON CONFLICT (tag_name, owner_id)
        DO NOTHING
        ;",
        tag_name,
        owner_id
    )
    .execute(&pool)
    .await?;

    let tag_id = utils::fetch_tag_id_with_pool(user, tag_name, &pool).await?;

    let channel_id = channel.to_string();
    let _query_cl = sqlx::query!(
        "INSERT INTO channel_list (channel_id, tag_id) VALUES ($1,$2)
        ON CONFLICT (channel_id, tag_id)
        DO NOTHING;",
        channel_id,
        tag_id
    )
    .execute(&pool)
    .await?;

    Ok(())
}
pub async fn unregister_channel(
    tag_name: &str,
    _channel: SlackChannelId,
    user: SlackUserId,
) -> anyhow::Result<()> {
    let db_url = super::db_url()?;
    let pool = SqlitePool::connect(&db_url).await?;
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
    let db_url = super::db_url()?;
    let pool = SqlitePool::connect(&db_url).await?;
    retrieve_bot_with_pool(tag_name, user, retrieve_bot, pool).await
}
async fn retrieve_bot_with_pool(
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

pub async fn is_valid_tag_for_user(user: &SlackUserId, tag_name: &str) -> anyhow::Result<bool> {
    let db_url = super::db_url()?;
    let pool = SqlitePool::connect(&db_url).await?;
    is_valid_tag_for_user_with_pool(user, tag_name, pool).await
}

async fn is_valid_tag_for_user_with_pool(
    user: &SlackUserId,
    tag_name: &str,
    pool: Pool<Sqlite>,
) -> anyhow::Result<bool> {
    let user_str = user.to_string();

    let is_valid = sqlx::query!(
        "
    SELECT EXISTS (
        SELECT 1
        FROM user_folder
        WHERE owner_id = $1 AND tag_name = $2
    ) AS is_exist
    ",
        user_str,
        tag_name
    )
    .fetch_one(&pool)
    .await?
    .is_exist
    .eq(&1);

    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use slack_morphism::SlackUserId;

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
        let (tag_name, _channel, user) = register_test(pool.clone()).await?;

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
        let (tag_name, _channel, user) = register_test(pool.clone()).await?;

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
    #[sqlx::test(migrations = "./migrations")]
    async fn test_is_valid(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag_name, _channel, user) = register_test(pool.clone()).await?;
        let not_auth_user = SlackUserId::new("U000".to_string());

        let is_valid = is_valid_tag_for_user_with_pool(&user, &tag_name, pool.clone()).await?;
        let _not_valid = is_valid_tag_for_user_with_pool(&not_auth_user, &tag_name, pool).await?;

        assert!(is_valid);
        Ok(())
    }
}
