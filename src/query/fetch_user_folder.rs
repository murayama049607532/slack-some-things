use slack_morphism::{SlackChannelId, SlackUserId};
use sqlx::{Pool, Sqlite, SqlitePool};

use super::DB_URL;

pub async fn tag_list_user(owner_id: SlackUserId) -> anyhow::Result<Vec<String>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    tag_list_user_with_pool(owner_id, &pool).await
}
async fn tag_list_user_with_pool(
    owner_id: SlackUserId,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<Vec<String>> {
    let owner_id_str = owner_id.to_string();

    let tag_list = sqlx::query!(
        "
    SELECT tag_name
    FROM user_folder
    WHERE owner_id = $1
    ",
        owner_id_str
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.tag_name.clone())
    .collect::<Vec<_>>();

    Ok(tag_list)
}
pub async fn tag_list_pub() -> anyhow::Result<Vec<String>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    tag_list_public_with_pool(&pool).await
}
async fn tag_list_public_with_pool(pool: &Pool<Sqlite>) -> anyhow::Result<Vec<String>> {
    let tag_list = sqlx::query!(
        "
    SELECT tag_name
    FROM user_folder
    WHERE owner_id = 'public'
    "
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.tag_name)
    .collect::<Vec<_>>();

    Ok(tag_list)
}

pub async fn channel_list(tag: &str, owner_id: SlackUserId) -> anyhow::Result<Vec<SlackChannelId>> {
    let pool = SqlitePool::connect(DB_URL).await?;
    channel_list_with_pool(tag, owner_id, &pool).await
}
async fn channel_list_with_pool(
    tag: &str,
    owner_id: SlackUserId,
    pool: &Pool<Sqlite>,
) -> anyhow::Result<Vec<SlackChannelId>> {
    let owner_id_str = owner_id.to_string();

    let ch_list = sqlx::query!(
        "
    SELECT channel_id
    FROM channel_list cl INNER JOIN user_folder uf
    ON cl.tag_id = uf.tag_id
    WHERE uf.owner_id = $1 AND uf.tag_name = $2",
        owner_id_str,
        tag
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| SlackChannelId::new(r.channel_id))
    .collect::<Vec<_>>();

    Ok(ch_list)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_tag_list(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let owner_id = SlackUserId::new("U00001".to_string());

        let tag_list_user = tag_list_user_with_pool(owner_id, &pool).await?;
        let tag_list_pub = tag_list_public_with_pool(&pool).await?;

        let desired_tag_list_user = vec!["test_a".to_string(), "test_b".to_string()];
        let desired_tag_list_pub = vec!["test_pub".to_string()];

        let contains_user = desired_tag_list_user
            .iter()
            .all(|s| tag_list_user.contains(s));
        let contains_pub = desired_tag_list_pub
            .iter()
            .all(|s| tag_list_pub.contains(s));

        assert!(contains_pub);
        assert!(contains_user);

        Ok(())
    }
    #[sqlx::test(migrations = "./migrations")]
    async fn test_channel_list(pool: Pool<Sqlite>) -> anyhow::Result<()> {
        let owner_id = SlackUserId::new("U00001".to_string());
        let owner_id_2 = SlackUserId::new("U00002".to_string());
        let public = SlackUserId::new("public".to_string());

        let tag_name = "test_a";

        let ch_list = channel_list_with_pool(tag_name, owner_id, &pool).await?;
        let ch_list_no_auth = channel_list_with_pool(tag_name, owner_id_2, &pool).await?;
        let ch_list_pub = channel_list_with_pool("test_pub", public, &pool).await?;

        let desired_ch_list = vec!["C01".to_string(), "C02".to_string()];

        let is_contain = desired_ch_list
            .iter()
            .all(|s| ch_list.contains(&SlackChannelId::new(s.to_string())));
        let is_contain_pub = ch_list_pub.contains(&SlackChannelId::new("C03".to_string()));

        assert!(ch_list_no_auth.is_empty());
        assert!(is_contain);
        assert!(is_contain_pub);

        Ok(())
    }
}
