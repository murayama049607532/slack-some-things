use futures::StreamExt;
use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};
use std::{str::SplitWhitespace, sync::Arc};

use anyhow::{anyhow, Context};

use crate::{create_channel, dist_target_map::channel_list_folder, set_target_tags, utils};

pub async fn add_command(mut args_iter: SplitWhitespace<'_>) -> anyhow::Result<(&str, Vec<&str>)> {
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

    Ok((tag, channels))
}

pub async fn delete_command(
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<(&str, Vec<&str>)> {
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
    Ok((tag, channels))
}

pub async fn set_command(
    args_iter: SplitWhitespace<'_>,
    channel_id_command: &SlackChannelId,
) -> anyhow::Result<Vec<String>> {
    let tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    set_target_tags::set_targets(channel_id_command, &tags).await?;
    Ok(tags)
}

pub async fn create_command(
    cli: Arc<SlackHyperClient>,
    mut args_iter: SplitWhitespace<'_>,
    user_id_command: SlackUserId,
) -> anyhow::Result<Vec<String>> {
    let channel_name = args_iter.next().context("argument error")?.to_string();
    let tags = args_iter
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    create_channel::create_retrieve_tags_channel(cli, &tags, channel_name, user_id_command).await?;
    Ok(tags)
}

pub async fn ch_list_command(
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<(&str, Vec<String>)> {
    let tag = args_iter.next().context("argument error")?;
    let ch_id_list = channel_list_folder::get_channel_list(tag).await?;
    let ch_name_list = ch_id_list
        .iter()
        .map(utils::channel_id_to_channel_name)
        .collect::<Vec<_>>();
    Ok((tag, ch_name_list))
}

pub async fn retreieve_bot_command(mut args_iter: SplitWhitespace<'_>) -> anyhow::Result<bool> {
    let tag = args_iter.next().context("argument error")?;
    let do_retrieve_bot_str = args_iter.next().context("argument error")?;
    let do_retrieve_bot = match do_retrieve_bot_str {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow!("argument should true or false")),
    }?;
    channel_list_folder::change_retrieve_bot(tag, do_retrieve_bot).await?;
    Ok(do_retrieve_bot)
}
