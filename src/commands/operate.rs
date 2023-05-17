use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;
use futures::{StreamExt, TryStreamExt};
use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};

use crate::{
    dist_target_map::{
        operate_folder::{self, FolderOperation},
        user_folders,
    },
    post_message::MessagePoster,
    utils,
};

async fn operate_ch_args(
    ch_list: SplitWhitespace<'_>,
    owner_id: SlackUserId,
    operation: FolderOperation,
    tag: String,
) -> anyhow::Result<()> {
    let ch_id_list = ch_list.map(utils::channel_preprocess);
    let channel_stream = futures::stream::iter(ch_id_list);
    channel_stream
        .map(|channel_id| async {
            operate_folder::operate_channel_list(
                tag.clone(),
                channel_id?,
                owner_id.clone(),
                operation.clone(),
                None,
            )
            .await
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
            (tag, SlackUserId::new(user_folders::PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let channels = args_iter.clone().collect::<Vec<_>>();

    operate_ch_args(args_iter, owner_add, FolderOperation::Add, tag.to_string()).await?;
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
            (tag, SlackUserId::new(user_folders::PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let channels = args_iter.clone().collect::<Vec<_>>();

    operate_ch_args(
        args_iter,
        owner_id,
        FolderOperation::Delete,
        tag.to_string(),
    )
    .await?;

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
            (tag, SlackUserId::new(user_folders::PUBLIC_TAGS.to_string()))
        }
        tag => (tag, user_id_command.clone()),
    };

    let do_retrieve_bot_str = args_iter.next().context("argument error")?;
    let do_retrieve_bot = match do_retrieve_bot_str {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow::anyhow!("argument should be true or false")),
    }?;

    operate_folder::operate_channel_list(
        tag.to_string(),
        channel_id_command.clone(),
        owner_id,
        FolderOperation::RetrieveBot,
        Some(do_retrieve_bot),
    )
    .await?;

    let retrieve_or_ignore = if do_retrieve_bot { "収集" } else { "無視" };
    let retreieve_bot_text =
        format!("以降、このタグはボットによるメッセージを{retrieve_or_ignore}します。");
    let _ = MessagePoster::new(channel_id_command, retreieve_bot_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
