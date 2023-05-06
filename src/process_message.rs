use anyhow::Context;
use regex::Regex;

use slack_morphism::{
    prelude::{SlackApiChatPostMessageRequest, SlackMessageEvent},
    SlackChannelId, SlackMessageContent,
};

use crate::slack_sender::SenderProfile;

pub fn message_event_to_req(
    msg_eve: SlackMessageEvent,
    channel_to: SlackChannelId,
    sender_profile: SenderProfile,
) -> anyhow::Result<SlackApiChatPostMessageRequest> {
    let raw_content = msg_eve.content.context("cannot get message content")?;
    let channel_from = msg_eve
        .origin
        .channel
        .context("cannot specify where the message from")?;
    let new_content = process_message(&raw_content, &channel_from)?;

    let post_req = SlackApiChatPostMessageRequest::new(channel_to, new_content)
        .with_icon_url(sender_profile.icon_url.to_string())
        .with_username(sender_profile.name);

    Ok(post_req)
}

fn process_message(
    msg_content: &SlackMessageContent,
    channel_from: &SlackChannelId,
) -> anyhow::Result<SlackMessageContent> {
    let raw_txt = msg_content.text.clone().context("cannot get message")?;
    let raw_txt_no_mention = escape_mention(&raw_txt)?;
    let new_txt = format!(" `<#{channel_from}>` {raw_txt_no_mention}");
    let new_content = msg_content.clone().with_text(new_txt).without_blocks();

    Ok(new_content)
}

fn escape_mention(txt: &str) -> anyhow::Result<String> {
    let re = Regex::new(r"<@([A-Z0-9]+)>")?;
    let new_txt = re
        .replace_all(txt, |caps: &regex::Captures| {
            let id = caps
                .get(1)
                .map_or(String::new(), |s| format!("{}{}", "@", s.as_str()));
            id
        })
        .to_string();
    Ok(new_txt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn escape_mention_test() {
        let test_txt = "test mention <@U12345T435T> test";
        let new = escape_mention(test_txt).unwrap();
        assert_eq!("test mention @U12345T435T test".to_string(), new);
    }
}
