use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{FromRow, SqlitePool};

use super::DB_URL;

#[derive(Default, Clone, FromRow, Debug)]
struct Dist {
    user_id: String,
    tag_id: i64,
    dist_channel_id: String,
}
async fn add(dist: SlackChannelId, user: SlackUserId, tag: &str) -> anyhow::Result<()> {
    add_with_url(dist, user, tag, DB_URL).await
}
async fn add_with_url(
    dist: SlackChannelId,
    user: SlackUserId,
    tag: &str,
    db_url: &str,
) -> anyhow::Result<()> {
    let db = SqlitePool::connect(db_url).await?;

    let user = user.to_string();
    let dist = dist.to_string();

    let _query = sqlx::query!(
        "INSERT INTO dist (user_id,tag_id, dist_channel_id) VALUES ($1, $2, $3);",
        user,
        tag,
        dist
    )
    .execute(&db)
    .await?;

    Ok(())
}
async fn remove_tag(dist: SlackChannelId, user: SlackUserId, tag: String) -> anyhow::Result<()> {
    Ok(())
}
async fn remove_tag_with_url(
    dist: SlackChannelId,
    user: SlackUserId,
    tag: String,
    db_url: &str,
) -> anyhow::Result<()> {
    let db = SqlitePool::connect(db_url).await?;

    let user = user.to_string();

    let _query = sqlx::query!(
        "DELETE FROM dist WHERE user_id = $1 AND tag_id = $2",
        user,
        tag
    )
    .bind(user.to_string())
    .bind(tag.to_string())
    .bind(dist.to_string())
    .execute(&db)
    .await?;

    Ok(())
}
async fn fetch() -> anyhow::Result<Vec<Dist>> {
    Ok(Vec::new())
}

async fn tag_list(dist: SlackChannelId) -> anyhow::Result<Vec<String>> {
    Ok(Vec::new())
}

async fn dist_list() -> anyhow::Result<Vec<SlackChannelId>> {
    Ok(Vec::new())
}

// return tags dist channel collects
async fn target_list(dist: &SlackChannelId) -> anyhow::Result<Vec<String>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add() {}

    #[tokio::test]
    async fn test_remove() {}
    #[tokio::test]
    async fn test_tag_list() {}
    #[tokio::test]
    async fn test_dists_list() {}
    #[tokio::test]
    async fn test_target_list() {}
}
