use anyhow::Context as _;
use dotenv::dotenv;
use regex::Regex;
use slack_morphism::{
    SlackApiToken, SlackApiTokenType, SlackApiTokenValue, SlackBotId, SlackChannelId,
};
use std::{env, path::Path};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

pub fn get_token(token_type: &SlackApiTokenType) -> anyhow::Result<SlackApiToken> {
    dotenv().ok();
    let token_key = match token_type {
        SlackApiTokenType::App => "SLACK_APP_TOKEN",
        SlackApiTokenType::Bot => "SLACK_BOT_TOKEN",
        SlackApiTokenType::User => "SLACK_USER_TOKEN",
    };
    let token_value: SlackApiTokenValue = env::var(token_key).context("Token is missing.")?.into();
    let app_token = SlackApiToken::new(token_value);
    Ok(app_token)
}

pub fn get_self_bot_id() -> anyhow::Result<SlackBotId> {
    dotenv().ok();
    let bot_id = env::var("SLACK_BOT_ID").context("my id is missing")?;
    Ok(SlackBotId(bot_id))
}

pub async fn update_json(json_path: &Path, content: String) -> anyhow::Result<()> {
    let backup_filename = format!(
        "backup_{}",
        json_path.file_stem().unwrap().to_str().unwrap()
    );
    let backup_path = Path::new(&backup_filename);
    let mut backup_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(backup_path)
        .await?;
    backup_file.write_all(content.as_bytes()).await?;

    tokio::fs::rename(backup_path, json_path).await?;
    Ok(())
}

pub fn channel_preprocess(channel: &str) -> anyhow::Result<SlackChannelId> {
    let channel_id_str = Regex::new(r"<#([^|]+)\|")?
        .captures(channel)
        .map_or("", |caps| caps.get(1).map_or("", |s| s.as_str()));
    Ok(SlackChannelId(channel_id_str.to_string()))
}
