mod create_channel;
mod set_target_tags;

use futures::StreamExt;
use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};
use std::{str::SplitWhitespace, sync::Arc};

use anyhow::{anyhow, Context};

use crate::{dist_target_map::channel_list_folder, post_message::MessagePoster, utils};

pub async fn add_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let tag = args_iter.next().context("error")?;
    let channels = args_iter.clone().collect::<Vec<_>>();
    let channel_stream = futures::stream::iter(args_iter);
    channel_stream
        .for_each(|channel| async {
            let channel_id =
                utils::channel_preprocess(channel).unwrap_or(SlackChannelId(String::new()));
            channel_list_folder::add_channel_list(tag, channel_id)
                .await
                .unwrap_or(());
        })
        .await;
    let add_text = format!("タグ {tag} に {channels:#?} が追加されました");
    let _ = MessagePoster::new(channel_id_command, add_text, cli)
        .post_message()
        .await?;

    Ok(())
}

pub async fn delete_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let tag = args_iter.next().context("error")?;
    let channels = args_iter.clone().collect::<Vec<_>>();
    let channel_stream = futures::stream::iter(args_iter);
    channel_stream
        .for_each(|channel| async {
            let channel_id =
                utils::channel_preprocess(channel).unwrap_or(SlackChannelId(String::new()));
            channel_list_folder::delete_channel_list(tag, channel_id)
                .await
                .unwrap_or(());
        })
        .await;
    let delete_text = format!("タグ {tag} から {channels:#?} が削除されました");
    let _ = MessagePoster::new(channel_id_command, delete_text, cli)
        .post_message()
        .await?;
    Ok(())
}

pub async fn set_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    set_target_tags::set_targets(&channel_id_command, &tags).await?;
    let set_text = format!(
        "以降、本チャンネルは以下のタグに登録されたチャンネルのメッセージを収集します。{tags:#?}"
    );
    let _ = MessagePoster::new(channel_id_command, set_text, cli)
        .post_message()
        .await?;
    Ok(())
}

pub async fn create_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let channel_name = args_iter.next().context("argument error")?.to_string();
    let tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let new_channel_id = create_channel::create_retrieve_tags_channel(
        cli.clone(),
        &tags,
        channel_name,
        user_id_command.clone(),
    )
    .await?;
    let create_text = format!("以下のタグに登録されたメッセージを収集する新しいチャンネル <#{new_channel_id}> を作成しました:{tags:#?}");
    let _ = MessagePoster::new(channel_id_command, create_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}

pub async fn retreieve_bot_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let tag = args_iter.next().context("argument error")?;
    let do_retrieve_bot_str = args_iter.next().context("argument error")?;
    let do_retrieve_bot = match do_retrieve_bot_str {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow!("argument should be true or false")),
    }?;
    channel_list_folder::change_retrieve_bot(tag, do_retrieve_bot).await?;
    let retrieve_or_ignore = if do_retrieve_bot { "収集" } else { "無視" };
    let retreieve_bot_text =
        format!("以降、このタグはボットによるメッセージを{retrieve_or_ignore}します。");
    let _ = MessagePoster::new(channel_id_command, retreieve_bot_text, cli)
        .post_message()
        .await?;
    Ok(())
}

pub async fn tag_list_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
) -> anyhow::Result<()> {
    let tags = channel_list_folder::get_tag_list().await;
    let tag_list_text = format!("タグのリストは以下です。 {tags:#?}");
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
    let tag = args_iter.next().context("argument error")?;
    let ch_id_list = channel_list_folder::get_channel_list(tag).await?;
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
