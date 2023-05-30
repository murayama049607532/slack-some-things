pub mod create_channel;
pub mod help;
pub mod operate;
pub mod set_target_tags;

use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};
use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;

use crate::{
    post_message::MessagePoster,
    query::{dist, fetch_user_folder},
    utils,
};

const PUBLIC_TAGS: &str = "public";

pub async fn tag_list_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
) -> anyhow::Result<()> {
    let user_tags = fetch_user_folder::tag_list_user(user_id_command.clone()).await?;
    let public_tags = fetch_user_folder::tag_list_pub().await?;

    let tag_list_text =
        format!("タグのリストは以下です。\n user: {user_tags:#?}\n public: {public_tags:#?}");
    let _ = MessagePoster::new(channel_id_command, tag_list_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}

pub async fn ch_list_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (tag, _owner_id) = match first_arg {
        "--public" => {
            let tag = args_iter.next().context("argument error")?;
            (tag, SlackUserId::new(PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let ch_id_list = fetch_user_folder::channel_list(tag, user_id_command.clone()).await?;
    let ch_name_list = ch_id_list
        .iter()
        .map(utils::channel_id_to_channel_name)
        .collect::<Vec<_>>();
    let ch_list_text = format!("タグ {tag} に登録されたチャンネルは以下です。 {ch_name_list:#?}");
    let _ = MessagePoster::new(channel_id_command, ch_list_text, cli)
        .post_ephemeral(user_id_command)
        .await?;

    Ok(())
}

pub async fn target_list_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
) -> anyhow::Result<()> {
    let target_list = dist::target_list(&channel_id_command).await?;
    let target_list_text = format!(
        "現在このチャンネルが収集対象としているタグのリストは以下です。\n {target_list:#?}"
    );
    let _ = MessagePoster::new(channel_id_command, target_list_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}

pub async fn undefined_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
) -> anyhow::Result<()> {
    let undefined_text = "このコマンド引数は未定義です。".to_string();
    let _ = MessagePoster::new(channel_id_command, undefined_text, cli)
        .post_ephemeral(user_id_command)
        .await?;

    Ok(())
}
