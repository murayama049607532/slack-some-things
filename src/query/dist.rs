use anyhow::Context;
use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};

use super::{
    utils::{self, fetch_tag_name_with_pool},
    DB_URL,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagOwner {
    tag: String,
    owner: SlackUserId,
}
impl TagOwner {
    pub fn new(tag: String, owner: SlackUserId) -> Self {
        Self { tag, owner }
    }
    pub fn get_tag(&self) -> &str {
        &self.tag
    }
    pub fn get_owner(&self) -> &SlackUserId {
        &self.owner
    }
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
        "INSERT INTO dist (user_id,tag_id, dist_channel_id) VALUES ($1, $2, $3);",
        user_str,
        tag_id,
        dist_str
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

pub async fn tag_owner_list(dist: &SlackChannelId) -> anyhow::Result<Vec<TagOwner>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    tag_owner_list_with_pool(dist, &pool).await
}
async fn tag_owner_list_with_pool(
    dist: &SlackChannelId,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<Vec<TagOwner>> {
    let dist_str = dist.to_string();

    let tag_owner_list = sqlx::query!(
        "
    SELECT uf.tag_name, uf.owner_id
    FROM dist INNER JOIN user_folder AS uf
    ON dist.tag_id = uf.tag_id
    WHERE dist.dist_channel_id = $1
        ",
        dist_str
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| {
        let owner_id = SlackUserId::new(r.owner_id);
        TagOwner::new(r.tag_name, owner_id)
    })
    .collect::<Vec<_>>();

    Ok(tag_owner_list)
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

#[allow(clippy::module_name_repetitions)]
pub async fn dist_list() -> anyhow::Result<Vec<SlackChannelId>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    dist_list_with_pool(pool).await
}

#[allow(clippy::module_name_repetitions)]
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

        let tag_list = target_list_with_pool(&dist, pool).await?;
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
    #[sqlx::test(migrations = "./migrations")]

    async fn test_tag_owner_list(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let (tag, dist, user) = add_test(pool.clone()).await?;

        let tag_owner_list = tag_owner_list_with_pool(&dist, &pool).await?;

        let tag_owner = TagOwner::new(tag, user);
        let is_contains = tag_owner_list.contains(&tag_owner);

        assert!(is_contains);

        Ok(())
    }
}
