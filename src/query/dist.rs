use anyhow::Context;
use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{Pool, Sqlite, SqlitePool};

use super::{
    utils::{self},
    DB_URL,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagOwner {
    tag: String,
    owner: SlackUserId,
}

pub async fn add_tag(dist: SlackChannelId, user: SlackUserId, tag: &str) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(DB_URL).await?;
    add_tag_with_pool(dist, user, tag, pool).await
}
async fn add_tag_with_pool(
    dist: SlackChannelId,
    user: SlackUserId,
    tag: &str,
    pool: Pool<Sqlite>,
) -> anyhow::Result<()> {
    let user_str = user.to_string();
    let dist_str = dist.to_string();

    let tag_id = utils::fetch_tag_id_with_pool(user, tag, &pool)
        .await
        .context("failed to fetch the tag")?;

    let _query = sqlx::query!(
        "INSERT INTO dist (user_id,tag_id, dist_channel_id) VALUES ($1, $2, $3);
        UPDATE user_folder
        SET valid_count = valid_count + 1
        WHERE tag_id = $4
        ",
        user_str,
        tag_id,
        dist_str,
        tag_id
    )
    .execute(&pool)
    .await?;

    Ok(())
}
pub async fn remove_tag(dist: SlackChannelId, user: SlackUserId, tag: &str) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(DB_URL).await?;
    remove_tag_with_pool(dist, user, tag, pool).await
}
async fn remove_tag_with_pool(
    _dist: SlackChannelId,
    user: SlackUserId,
    tag: &str,
    pool: Pool<Sqlite>,
) -> anyhow::Result<()> {
    let user_str = user.to_string();

    let tag_id = utils::fetch_tag_id_with_pool(user, tag, &pool)
        .await
        .context("failed to fetch the tag")?;

    let _query = sqlx::query!(
        "DELETE FROM dist WHERE user_id = $1 AND tag_id = $2;
        UPDATE user_folder
        SET valid_count = valid_count - 1
        WHERE tag_id = $3",
        user_str,
        tag_id,
        tag_id
    )
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn target_list(dist: &SlackChannelId) -> anyhow::Result<Vec<String>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    target_list_with_pool(dist, pool).await
}

async fn target_list_with_pool(
    dist: &SlackChannelId,
    pool: Pool<Sqlite>,
) -> anyhow::Result<Vec<String>> {
    let dist_str = dist.to_string();

    let target_list = sqlx::query!(
        "
    SELECT uf.tag_name
    FROM dist INNER JOIN user_folder AS uf
    ON dist.tag_id = uf.tag_id
    WHERE dist.tag_id = uf.tag_id AND dist.dist_channel_id = $1
    ",
        dist_str
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|r| r.tag_name)
    .collect::<Vec<_>>();

    Ok(target_list)
}

#[cfg(test)]
mod tests {
    use sqlx::{Pool, Sqlite};

    use super::*;

    async fn add_test(pool: Pool<Sqlite>) -> anyhow::Result<(String, SlackChannelId, SlackUserId)> {
        let tag_name = "test_dist";
        let channel = SlackChannelId::new("C012345dist".to_string());
        let user = SlackUserId::new("U0987654".to_string());
        add_tag_with_pool(channel.clone(), user.clone(), tag_name, pool).await?;
        Ok((tag_name.to_string(), channel, user))
    }

    #[sqlx::test(migrations = "./migrations")]

    async fn test_add_tag(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (_tag, dist, user) = add_test(pool.clone()).await?;
        let user_id_str = user.to_string();

        let dist_fetch = sqlx::query!(
            "
        SELECT dist_channel_id
        FROM dist
        WHERE user_id = $1
        ",
            user_id_str,
        )
        .fetch_one(&pool)
        .await?
        .dist_channel_id;

        assert_eq!(dist_fetch, dist.to_string());

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]

    async fn test_remove(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag, dist, user) = add_test(pool.clone()).await?;
        let user_id_str = user.to_string();

        remove_tag_with_pool(dist.clone(), user, &tag, pool.clone()).await?;

        let fail_fetch = sqlx::query!(
            "
        SELECT dist_channel_id
        FROM dist
        WHERE user_id = $1 
        ",
            user_id_str,
        )
        .fetch_one(&pool)
        .await
        .is_err();

        assert!(fail_fetch);

        Ok(())
    }
    #[sqlx::test(migrations = "./migrations")]

    async fn test_tag_list(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag, dist, _user) = add_test(pool.clone()).await?;

        let tag_list = target_list_with_pool(&dist, pool).await?;
        let is_contains = tag_list.contains(&tag);

        assert!(is_contains);

        Ok(())
    }
}
