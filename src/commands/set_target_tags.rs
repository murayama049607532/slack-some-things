use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;
use futures::{future, StreamExt, TryStreamExt};

use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};

use crate::{
    post_message::MessagePoster,
    query::{self, dist, user_folder::is_valid_tag_for_user},
};

pub async fn set_targets(
    channel_id: &SlackChannelId,
    user_command: SlackUserId,
    tags: &[String],
    set: bool,
) -> anyhow::Result<Vec<String>> {
    let tags_stream = futures::stream::iter(tags);

    let authed_tags = tags_stream
        .map(|tag| async {
            let is_valid = is_valid_tag_for_user(&user_command, tag).await?;
            if is_valid {
                anyhow::Ok(tag.clone())
            } else {
                Err(anyhow::anyhow!("the tag does not exist"))
            }
        })
        .then(|s| s)
        .try_collect::<Vec<_>>()
        .await?;

    let authed_tags_stream = futures::stream::iter(authed_tags.iter());

    let tags = authed_tags_stream
        .map(|tag| async {
            if set {
                dist::add_tag(channel_id.clone(), user_command.clone(), tag).await?;
            } else {
                dist::remove_tag(channel_id.clone(), user_command.clone(), tag).await?;
            };
            anyhow::Ok(tag.clone())
        })
        .then(|s| s)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(tags)
}

pub async fn set_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (head_tag, owner_id) = match first_arg {
        "--public" => (None, SlackUserId::new(super::PUBLIC_TAGS.to_string())),
        head_tag => (Some(head_tag), user_id_command.clone()),
    };

    let mut tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    if let Some(head) = head_tag {
        tags.insert(0, head.to_string());
    };

    let set_tags = set_targets(&channel_id_command, owner_id, &tags, true).await?;
    let set_text = format!(
        "以降、本チャンネルは以下のタグに登録されたチャンネルのメッセージを収集します。{set_tags:#?}"
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
        "--public" => (None, SlackUserId::new(super::PUBLIC_TAGS.to_string())),
        head_tag => (Some(head_tag), user_id_command.clone()),
    };

    let mut tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    if let Some(head) = head_tag {
        tags.insert(0, head.to_string());
    };

    let set_tags = set_targets(&channel_id_command, owner_id, &tags, false).await?;
    let set_text =
        format!("以下のタグに登録されたチャンネルのメッセージの収集を停止します。{set_tags:#?}");
    let _ = MessagePoster::new(channel_id_command, set_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
