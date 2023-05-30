use anyhow::Context as _;
use dotenvy::dotenv;
use regex::Regex;
use slack_morphism::{
    SlackApiToken, SlackApiTokenType, SlackApiTokenValue, SlackBotId, SlackChannelId,
};
use std::env;

pub fn get_token(token_type: &SlackApiTokenType) -> anyhow::Result<SlackApiToken> {
    dotenv().ok();
    let token_key = match token_type {
        SlackApiTokenType::App => "SLACK_APP_TOKEN",
        SlackApiTokenType::Bot => "SLACK_BOT_TOKEN",
        SlackApiTokenType::User => "SLACK_USER_TOKEN",
    };
    let token_value: SlackApiTokenValue = env::var(token_key).context("token is missing.")?.into();
    let app_token = SlackApiToken::new(token_value);
    Ok(app_token)
}

pub fn get_self_bot_id() -> anyhow::Result<SlackBotId> {
    dotenv().ok();
    let bot_id = env::var("SLACK_BOT_ID").context("bot id is missing")?;
    Ok(SlackBotId(bot_id))
}

pub fn channel_preprocess(channel: &str) -> anyhow::Result<SlackChannelId> {
    let channel_id_str = Regex::new(r"<#([^|]+)\|")
        .unwrap()
        .captures(channel)
        .and_then(|caps| caps.get(1).map(|s| s.as_str()))
        .context("validation error")?;
    Ok(SlackChannelId(channel_id_str.to_string()))
}

pub fn channel_id_to_channel_name(channel_id: &SlackChannelId) -> String {
    let raw_channel_id = &channel_id.0;
    let channel_name = format!("<#{raw_channel_id}>");
    channel_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn channel_preprocess_test() {
        let slack_channel = "<#test_channel|https://xxxxxxxxxxxxxxxx>";
        let processed = channel_preprocess(slack_channel).unwrap();
        assert_eq!("test_channel", processed.0);
    }
    #[tokio::test]
    async fn channel_id_to_channel_name_test() {
        let slack_channel = SlackChannelId::new("C12345678".to_string());
        let processed = channel_id_to_channel_name(&slack_channel);
        assert_eq!("<#C12345678>", processed);
    }
}
