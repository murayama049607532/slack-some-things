use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;
use slack_morphism::{
    prelude::{
        SlackApiConversationsCreateRequest, SlackApiConversationsCreateResponse,
        SlackApiConversationsInviteRequest, SlackHyperClient,
    },
    SlackApiTokenType, SlackChannelId, SlackUserId,
};

use crate::{dist_target_map::user_folders, post_message::MessagePoster, utils};

use super::set_target_tags::set_targets;

async fn create_priv_channel(
    cli: Arc<SlackHyperClient>,
    channel_name: String,
) -> anyhow::Result<SlackApiConversationsCreateResponse> {
    let app_token = utils::get_token(&SlackApiTokenType::Bot)?;
    let session = cli.open_session(&app_token);
    let create_req = SlackApiConversationsCreateRequest::new(channel_name).with_is_private(true);
    let res = session.conversations_create(&create_req).await?;

    Ok(res)
}
async fn invite_user(
    cli: Arc<SlackHyperClient>,
    user_id: SlackUserId,
    channel_id: SlackChannelId,
) -> anyhow::Result<()> {
    let app_token = utils::get_token(&SlackApiTokenType::Bot)?;
    let session = cli.open_session(&app_token);
    let invite_req = SlackApiConversationsInviteRequest::new(channel_id, vec![user_id]);
    session.conversations_invite(&invite_req).await?;

    Ok(())
}
pub async fn create_retrieve_tags_channel(
    cli: Arc<SlackHyperClient>,
    tags: &[String],
    channel_name: String,
    user_id: SlackUserId,
    owner_id: SlackUserId,
) -> anyhow::Result<SlackChannelId> {
    let create_res = create_priv_channel(cli.clone(), channel_name).await?;
    let channel_id = create_res.channel.id;

    invite_user(cli.clone(), user_id, channel_id.clone())
        .await
        .context("failed to invite user to created channel")?;

    set_targets(&channel_id, owner_id, tags, true)
        .await
        .context("failed to set tags in created channel")?;
    let set_text = format!(
        "以降、本チャンネルは以下のタグに登録されたチャンネルのメッセージを収集します。{tags:#?}"
    );
    let _ = MessagePoster::new(channel_id.clone(), set_text, cli)
        .post_message()
        .await?;
    Ok(channel_id)
}

pub async fn create_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (channel_name, owner_id) = match first_arg {
        "--public" => (
            args_iter.next().context("argument error")?.to_string(),
            SlackUserId::new(user_folders::PUBLIC_TAGS.to_string()),
        ),
        channel => (channel.to_string(), user_id_command.clone()),
    };

    let tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let new_channel_id = create_retrieve_tags_channel(
        cli.clone(),
        &tags,
        channel_name,
        user_id_command.clone(),
        owner_id,
    )
    .await?;
    let create_text = format!("以下のタグに登録されたメッセージを収集する新しいチャンネル <#{new_channel_id}> を作成しました:{tags:#?}");
    let _ = MessagePoster::new(channel_id_command, create_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
