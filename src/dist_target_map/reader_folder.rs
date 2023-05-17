use std::collections::HashSet;

use anyhow::Context;
use slack_morphism::{SlackChannelId, SlackUserId};

use super::operate_folder;

pub async fn load_user_channel_list(
    tag: &str,
    user: SlackUserId,
) -> anyhow::Result<HashSet<SlackChannelId>> {
    let ch_list_folder = operate_folder::load_user_folders_json()
        .await?
        .user_ch_list_folders(&user);

    let ch_list = ch_list_folder
        .get_channel_list(tag)
        .context("the tag does not exist")?;
    Ok(ch_list)
}
pub async fn load_tag_list(user_id: SlackUserId) -> anyhow::Result<(Vec<String>, Vec<String>)> {
    let ch_list_folders = operate_folder::load_user_folders_json().await?;
    let user_tags = ch_list_folders.user_tag_list(&user_id);
    let public_tags = ch_list_folders.public_tag_list();

    Ok((user_tags, public_tags))
}
