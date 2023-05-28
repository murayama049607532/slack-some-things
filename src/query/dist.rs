use anyhow::Context;
use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};

use super::{
    utils::{self, fetch_tag_name_with_pool},
    DB_URL,
};

#[derive(Default, Clone, FromRow, Debug)]
struct Dist {
    user_id: String,
    tag_id: i64,
    dist_channel_id: String,
}
async fn add_tag(dist: SlackChannelId, user: SlackUserId, tag: &str) -> anyhow::Result<()> {
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
        "INSERT INTO dist (user_id,tag_id, dist_channel_id) VALUES ($1, $2, $3);",
        user_str,
        tag_id,
        dist_str
    )
    .execute(&pool)
    .await?;

    Ok(())
}
async fn remove_tag(dist: SlackChannelId, user: SlackUserId, tag: String) -> anyhow::Result<()> {
    Ok(())
}
async fn remove_tag_with_pool(
    dist: SlackChannelId,
    user: SlackUserId,
    tag: &str,
    pool: Pool<Sqlite>,
) -> anyhow::Result<()> {
    let user_str = user.to_string();

    let tag_id = utils::fetch_tag_id_with_pool(user, tag, &pool)
        .await
        .context("failed to fetch the tag")?;

    let _query = sqlx::query!(
        "DELETE FROM dist WHERE user_id = $1 AND tag_id = $2",
        user_str,
        tag_id
    )
    .execute(&pool)
    .await?;

    Ok(())
}
async fn fetch() -> anyhow::Result<Vec<Dist>> {
    Ok(Vec::new())
}

async fn target_list_with_pool(
    dist: SlackChannelId,
    pool: Pool<Sqlite>,
) -> anyhow::Result<Vec<String>> {
    let dist_str = dist.to_string();

    let target_list = sqlx::query!(
        "
    SELECT uf.tag_name
    FROM dist INNER JOIN user_folder AS uf
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

async fn dist_list_with_pool(pool: Pool<Sqlite>) -> anyhow::Result<Vec<SlackChannelId>> {
    let dist_list = sqlx::query!(
        "
    SELECT dist_channel_id
    FROM dist
    "
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|r| {
        let dist_id_str = r.dist_channel_id;
        SlackChannelId::new(dist_id_str)
    })
    .collect::<Vec<_>>();

    Ok(dist_list)
}

#[cfg(test)]
mod tests {
    use sqlx::{Pool, Sqlite};

    use crate::query::utils::fetch_tag_id_with_pool;

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
        let (tag, dist, user) = add_test(pool.clone()).await?;
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
        let (tag, dist, user) = add_test(pool.clone()).await?;

        let tag_list = target_list_with_pool(dist, pool).await?;
        let is_contains = tag_list.contains(&tag);

        assert!(is_contains);

        Ok(())
    }
    #[sqlx::test(migrations = "./migrations")]

    async fn test_dists_list(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag, dist, user) = add_test(pool.clone()).await?;

        let dist_list = dist_list_with_pool(pool).await?;
        let is_contains = dist_list.contains(&dist);

        assert!(is_contains);
        Ok(())
    }
}
