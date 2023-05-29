use slack_morphism::{prelude::SlackMessageEvent, SlackChannelId};
use sqlx::{Pool, Sqlite, SqlitePool};

use super::DB_URL;

// Determine if the channel is a collection target for the dist
pub async fn is_target(msg_event: SlackMessageEvent) -> anyhow::Result<bool> {
    let pool = SqlitePool::connect(DB_URL).await?;
    is_target_with_pool(msg_event, pool).await
}
async fn is_target_with_pool(
    msg_event: SlackMessageEvent,
    pool: Pool<Sqlite>,
) -> anyhow::Result<bool> {
    Ok(false)
}

// Return all dist channels that have set the tags that is registred the channel
pub async fn target_to_dists(target: SlackChannelId) -> anyhow::Result<Vec<SlackChannelId>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    target_to_dists_with_pool(target, &pool).await
}
pub async fn target_to_dists_with_pool(
    target: SlackChannelId,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<Vec<SlackChannelId>> {
    Ok(Vec::default())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_is_target(pool: Pool<Sqlite>) -> anyhow::Result<()> {}
}
