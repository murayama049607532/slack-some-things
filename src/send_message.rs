use std::sync::Arc;

use anyhow::Context;
use slack_morphism::{
    prelude::{
        SlackApiChatPostEphemeralRequest, SlackApiChatPostEphemeralResponse,
        SlackApiChatPostMessageRequest, SlackApiChatPostMessageResponse, SlackHyperClient,
    },
    SlackApiTokenType, SlackChannelId,
};

use crate::utils;
#[derive(Debug, Clone)]

pub enum SlackApiMessageRequest {
    PostMessage(SlackApiChatPostMessageRequest),
    PostEphemeral(SlackApiChatPostEphemeralRequest),
}
#[derive(Debug, Clone)]
pub enum SlackApiMessageResponse {
    PostMessage(SlackApiChatPostMessageResponse),
    PostEphemeral(SlackApiChatPostEphemeralResponse),
}

pub async fn send_message(
    cli: Arc<SlackHyperClient>,
    req: SlackApiMessageRequest,
) -> anyhow::Result<SlackApiMessageResponse> {
    let app_token = utils::get_token(&SlackApiTokenType::Bot)?;
    let session = cli.open_session(&app_token);
    let message_res: SlackApiMessageResponse = match req {
        SlackApiMessageRequest::PostMessage(req) => SlackApiMessageResponse::PostMessage(
            session
                .chat_post_message(&req)
                .await
                .context("failed to post message.")?,
        ),
        SlackApiMessageRequest::PostEphemeral(req) => SlackApiMessageResponse::PostEphemeral(
            session
                .chat_post_ephemeral(&req)
                .await
                .context("failed to post message.")?,
        ),
    };
    Ok(message_res)
}
