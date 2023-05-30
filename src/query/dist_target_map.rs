use std::collections::HashSet;

use slack_morphism::{SlackChannelId, SlackMessageSender};
use sqlx::{Pool, Sqlite, SqlitePool};

use super::DB_URL;

// Determine if the channel is a collection target
pub async fn is_target_for_some(
    channel_from: SlackChannelId,
    sender: SlackMessageSender,
) -> anyhow::Result<bool> {
    let pool = SqlitePool::connect(DB_URL).await?;
    is_target_for_some_with_pool(channel_from, sender, &pool).await
}
async fn is_target_for_some_with_pool(
    channel_from: SlackChannelId,
    sender: SlackMessageSender,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<bool> {
    let channel_str = channel_from.to_string();

    let is_bot = sender.bot_id.is_some();

    let self_bot = crate::utils::get_self_bot_id()?;
    let is_self = sender.bot_id.map_or(false, |bot_id| bot_id == self_bot);

    if is_self {
        return Ok(false);
    }

    let is_target = sqlx::query!(
        "
        SELECT cl.channel_id, uf.bot
        FROM channel_list cl INNER JOIN user_folder uf
        ON cl.tag_id = uf.tag_id
        WHERE cl.channel_id = $1
    ",
        channel_str
    )
    .fetch_all(pool)
    .await
    .map_or(false, |records| {
        records.iter().any(|rec| !is_bot || rec.bot)
    });

    Ok(is_target)
}

// Return all dist channels that have set the tags that is registred the channel
pub async fn target_to_dists(
    target: SlackChannelId,
    sender: SlackMessageSender,
) -> anyhow::Result<HashSet<SlackChannelId>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    target_to_dists_with_pool(target, sender, &pool).await
}
pub async fn target_to_dists_with_pool(
    target: SlackChannelId,
    sender: SlackMessageSender,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<HashSet<SlackChannelId>> {
    let channel_str = target.to_string();

    let is_bot = sender.bot_id.is_some();

    let dists = sqlx::query!(
        "
    SELECT dist.dist_channel_id, uf_valid.bot
    FROM dist 
    INNER JOIN (
        SELECT uf.tag_id, uf.bot
            FROM user_folder uf INNER JOIN channel_list cl
            ON uf.tag_id = cl.tag_id
            WHERE 0 < uf.valid_count AND cl.channel_id = $1
        ) AS uf_valid
    ON uf_valid.tag_id = dist.tag_id
    ",
        channel_str
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .filter(|r| !is_bot || r.bot)
    .map(|r| SlackChannelId::new(r.dist_channel_id))
    .collect::<HashSet<_>>();

    Ok(dists)
}

#[cfg(test)]
mod tests {

    use slack_morphism::{SlackBotId, SlackUserId};

    use crate::utils::get_self_bot_id;

    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_is_target(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let channel_from_1 = SlackChannelId::new("C01".to_string());
        let channel_from_2 = SlackChannelId::new("C02".to_string());

        let channel_from_not_collect = SlackChannelId::new("Cnot".to_string());

        let user = SlackUserId::new("Uanybody".to_string());
        let sender_user = SlackMessageSender::new().with_user(user);

        assert!(
            is_target_for_some_with_pool(channel_from_1.clone(), sender_user.clone(), &pool)
                .await?
        );
        assert!(!is_target_for_some_with_pool(channel_from_not_collect, sender_user, &pool).await?);

        let bot = SlackBotId::new("B01234567".to_string());
        let sender_bot = SlackMessageSender::new().with_bot_id(bot);

        assert!(
            !is_target_for_some_with_pool(channel_from_1.clone(), sender_bot.clone(), &pool)
                .await?
        );
        assert!(is_target_for_some_with_pool(channel_from_2, sender_bot, &pool).await?);

        let _bot_self_id = get_self_bot_id()?;
        let bot_self = SlackBotId::new("B01234567".to_string());
        let sender_self = SlackMessageSender::new().with_bot_id(bot_self);

        assert!(!is_target_for_some_with_pool(channel_from_1, sender_self, &pool).await?);

        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_target_to_dists(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let channel_from_1 = SlackChannelId::new("C01".to_string());
        let channel_from_2 = SlackChannelId::new("C02".to_string());

        let user = SlackUserId::new("Uanybody".to_string());
        let sender_user = SlackMessageSender::new().with_user(user);

        let bot = SlackBotId::new("B01234567".to_string());
        let sender_bot = SlackMessageSender::new().with_bot_id(bot);

        let dists_1 = target_to_dists_with_pool(channel_from_1, sender_user.clone(), &pool).await?;
        let dists_2 = target_to_dists_with_pool(channel_from_2.clone(), sender_bot, &pool).await?;
        let dists_1_2 = target_to_dists_with_pool(channel_from_2, sender_user, &pool).await?;

        let dist_ch = SlackChannelId::new("Cdist".to_string());
        let dist_bot_ch = SlackChannelId::new("Cdist_bot".to_string());

        let desired_dists_1 = vec![dist_ch.clone()];
        let desired_dists_2 = vec![dist_bot_ch.clone()];
        let desired_dists_1_2 = vec![dist_ch, dist_bot_ch];

        assert!(desired_dists_1.iter().all(|ch| dists_1.contains(ch)));
        assert!(desired_dists_2.iter().all(|ch| dists_2.contains(ch)));
        assert!(desired_dists_1_2.iter().all(|ch| dists_1_2.contains(ch)));

        Ok(())
    }
}
