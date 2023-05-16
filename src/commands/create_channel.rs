use std::sync::Arc;

use anyhow::Context;
use slack_morphism::{
    prelude::{
        SlackApiConversationsCreateRequest, SlackApiConversationsCreateResponse,
        SlackApiConversationsInviteRequest, SlackHyperClient,
    },
    SlackApiTokenType, SlackChannelId, SlackUserId,
};

use crate::{post_message::MessagePoster, utils};

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
) -> anyhow::Result<SlackChannelId> {
    let create_res = create_priv_channel(cli.clone(), channel_name).await?;
    let channel_id = create_res.channel.id;

    invite_user(cli.clone(), user_id.clone(), channel_id.clone())
        .await
        .context("failed to invite user to created channel")?;

    set_targets(&channel_id, user_id, tags)
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
