use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;
use futures::{StreamExt, TryStreamExt};

use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};

use crate::{
    dist_target_map::{self, channel_dist, user_folders},
    post_message::MessagePoster,
};

pub async fn set_targets(
    channel_id: &SlackChannelId,
    user_command: SlackUserId,
    tags: &[String],
    set: bool,
) -> anyhow::Result<()> {
    let user_folders = dist_target_map::operate_folder::load_user_folders_json().await?;
    let auth_tags_iter = tags
        .iter()
        .filter(|tag| user_folders.is_valid(&user_command, tag));

    let tags_stream = futures::stream::iter(auth_tags_iter);

    tags_stream
        .map(|tag| async {
            if set {
                channel_dist::add_dists_json(
                    channel_id.clone(),
                    user_command.clone(),
                    tag.to_string(),
                )
                .await
            } else {
                channel_dist::remove_dists_json(
                    channel_id.clone(),
                    user_command.clone(),
                    tag.to_string(),
                )
                .await
            }
        })
        .then(|s| s)
        .try_collect()
        .await?;
    Ok(())
}

pub async fn set_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (head_tag, owner_id) = match first_arg {
        "--public" => (
            None,
            SlackUserId::new(user_folders::PUBLIC_TAGS.to_string()),
        ),
        head_tag => (Some(head_tag), user_id_command.clone()),
    };

    let mut tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    if let Some(head) = head_tag {
        tags.insert(0, head.to_string());
    };

    set_targets(&channel_id_command, owner_id, &tags, true).await?;
    let set_text = format!(
        "以降、本チャンネルは以下のタグに登録されたチャンネルのメッセージを収集します。{tags:#?}"
    );
    let _ = MessagePoster::new(channel_id_command, set_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}

pub async fn unset_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (head_tag, owner_id) = match first_arg {
        "--public" => (
            None,
            SlackUserId::new(user_folders::PUBLIC_TAGS.to_string()),
        ),
        head_tag => (Some(head_tag), user_id_command.clone()),
    };

    let mut tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    if let Some(head) = head_tag {
        tags.insert(0, head.to_string());
    };

    set_targets(&channel_id_command, owner_id, &tags, false).await?;
    let set_text =
        format!("以下のタグに登録されたチャンネルのメッセージの収集を停止します。{tags:#?}");
    let _ = MessagePoster::new(channel_id_command, set_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
