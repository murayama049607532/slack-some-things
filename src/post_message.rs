use std::sync::Arc;

use anyhow::Context;
use rsb_derive::Builder;
use slack_morphism::{
    prelude::{
        SlackApiChatPostEphemeralRequest, SlackApiChatPostEphemeralResponse,
        SlackApiChatPostMessageRequest, SlackApiChatPostMessageResponse, SlackHyperClient,
    },
    SlackApiTokenType, SlackChannelId, SlackMessageContent, SlackUserId,
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
#[derive(Debug, Clone, Builder)]
pub struct MessagePoster {
    channel: SlackChannelId,
    text: String,
    cli: Arc<SlackHyperClient>,
}

impl MessagePoster {
    pub async fn post_message(&self) -> anyhow::Result<SlackApiMessageResponse> {
        let token = utils::get_token(&SlackApiTokenType::Bot)?;
        let session = self.cli.open_session(&token);
        let content = SlackMessageContent::new().with_text(self.text.clone());
        let req = SlackApiChatPostMessageRequest::new(self.channel.clone(), content);
        let message_res = SlackApiMessageResponse::PostMessage(
            session
                .chat_post_message(&req)
                .await
                .context("failed to post message.")?,
        );
        Ok(message_res)
    }
    pub async fn post_ephemeral(
        &self,
        user_id: SlackUserId,
    ) -> anyhow::Result<SlackApiMessageResponse> {
        let token = utils::get_token(&SlackApiTokenType::Bot)?;
        let session = self.cli.open_session(&token);
        let content = SlackMessageContent::new().with_text(self.text.clone());
        let req = SlackApiChatPostEphemeralRequest::new(self.channel.clone(), user_id, content);
        let message_res = SlackApiMessageResponse::PostEphemeral(
            session
                .chat_post_ephemeral(&req)
                .await
                .context("failed to post message.")?,
        );
        Ok(message_res)
    }
}

pub async fn send_req(
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
