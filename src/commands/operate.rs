use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;
use futures::{StreamExt, TryStreamExt};
use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};

use crate::{post_message::MessagePoster, query::user_folder, utils};

async fn operate_channel_list(
    channel_id: SlackChannelId,
    owner_id: SlackUserId,
    register: bool,
    tag: String,
) -> anyhow::Result<()> {
    if register {
        user_folder::register_channel(&tag, channel_id, owner_id).await?;
    } else {
        user_folder::unregister_channel(&tag, channel_id, owner_id).await?;
    }

    Ok(())
}

async fn operate_ch_args(
    ch_list: SplitWhitespace<'_>,
    owner_id: SlackUserId,
    register: bool,
    tag: String,
) -> anyhow::Result<()> {
    let ch_id_list = ch_list.map(utils::channel_preprocess);
    let channel_stream = futures::stream::iter(ch_id_list);
    channel_stream
        .map(|channel_id| async {
            operate_channel_list(channel_id?, owner_id.clone(), register, tag.clone()).await
        })
        .then(|s| s)
        .try_collect()
        .await?;
    Ok(())
}

pub async fn add_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (tag, owner_add) = match first_arg {
        "--public" => {
            let tag = args_iter.next().context("argument error")?;
            (tag, SlackUserId::new(super::PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let channels = args_iter.clone().collect::<Vec<_>>();

    operate_ch_args(args_iter, owner_add, true, tag.to_string()).await?;
    let add_text = format!("タグ {tag} に {channels:#?} が追加されました");
    let _ = MessagePoster::new(channel_id_command, add_text, cli)
        .post_ephemeral(user_id_command)
        .await?;

    Ok(())
}

pub async fn delete_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (tag, owner_id) = match first_arg {
        "--public" => {
            let tag = args_iter.next().context("argument error")?;
            (tag, SlackUserId::new(super::PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let channels = args_iter.clone().collect::<Vec<_>>();

    operate_ch_args(args_iter, owner_id, false, tag.to_string()).await?;

    let delete_text = format!("タグ {tag} から {channels:#?} が削除されました");
    let _ = MessagePoster::new(channel_id_command, delete_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
pub async fn retreieve_bot_command(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first_arg = args_iter.next().context("argument error")?;
    let (tag, owner_id) = match first_arg {
        "--public" => {
            let tag = args_iter.next().context("argument error")?;
            (tag, SlackUserId::new(super::PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let do_retrieve_bot_str = args_iter.next().context("argument error")?;
    let do_retrieve_bot = match do_retrieve_bot_str {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow::anyhow!("argument should be true or false")),
    }?;

    user_folder::retrieve_bot(tag, owner_id, do_retrieve_bot).await?;

    let retrieve_or_ignore = if do_retrieve_bot { "収集" } else { "無視" };
    let retreieve_bot_text =
        format!("以降、このタグはボットによるメッセージを{retrieve_or_ignore}します。");
    let _ = MessagePoster::new(channel_id_command, retreieve_bot_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
