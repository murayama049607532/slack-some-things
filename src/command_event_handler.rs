use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use slack_morphism::{
    prelude::{
        SlackApiChatPostMessageRequest, SlackClientEventsUserState, SlackCommandEvent,
        SlackCommandEventResponse, SlackHyperClient,
    },
    SlackChannelId, SlackMessageContent,
};

use crate::{
    create_channel::create_retrieve_tags_channel,
    dist_target_map::{channel_dist, channel_list_folder},
    set_target_tags, slack_sender, utils,
};

#[allow(clippy::too_many_lines)]
pub async fn command_event_handler(
    event: SlackCommandEvent,
    cli: Arc<SlackHyperClient>,
    _state: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    println!("{event:#?}");
    let channel_id_command = event.channel_id.clone();
    let user_id_command = event.user_id;

    let full = event.text.clone().unwrap_or(String::new());
    let mut args_iter = full.split_whitespace();
    let first_arg = args_iter.next().context("error")?;

    let cli_send_msg = cli.clone();
    let send_message = |msg_txt: String| async {
        let msg_req = SlackApiChatPostMessageRequest::new(
            channel_id_command.clone(),
            SlackMessageContent::new().with_text(msg_txt),
        );
        slack_sender::send_message_req(cli_send_msg, msg_req).await
    };

    match first_arg {
        "add" => {
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
            send_message(add_text).await?;
        }
        "delete" => {
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
            send_message(delete_text).await?;
        }
        "set" => {
            let tags = args_iter
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
            set_target_tags::set_targets(&channel_id_command, &tags).await?;
            let set_text = format!(
                "以降、本チャンネルは以下のタグに登録されたチャンネルのメッセージを収集します。{tags:#?}"
            );
            send_message(set_text).await?;
        }
        "create_channel" => {
            let channel_name = args_iter.next().context("argument error")?.to_string();
            let tags = args_iter
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>();
            create_retrieve_tags_channel(cli, &tags, channel_name, user_id_command).await?;
            let set_text = format!("test message:{tags:#?}");
            send_message(set_text).await?;
        }
        "retrieve_bot" => {
            let tag = args_iter.next().context("argument error")?;
            let do_retrieve_bot_str = args_iter.next().context("argument error")?;
            let do_retrieve_bot = do_retrieve_bot_str == "true";
            channel_list_folder::change_retrieve_bot(tag, do_retrieve_bot).await?;
            let retreieve_bot = if do_retrieve_bot {
                "以降、このタグはボットによるメッセージを収集します。"
            } else {
                "以降、このタグはボットによるメッセージを無視します。"
            };
            send_message(retreieve_bot.to_string()).await?;
        }
        "tag_list" => {
            let tags = channel_list_folder::get_tag_list().await?;
            let tag_list_message = format!("タグのリストは以下です。 {tags:#?}");
            send_message(tag_list_message).await?;
        }
        "ch_list" => {
            let tag = args_iter.next().context("argument error")?;
            let ch_id_list = channel_list_folder::get_channel_list(tag).await?;
            let ch_name_list = ch_id_list
                .iter()
                .map(utils::channel_id_to_channel_name)
                .collect::<Vec<_>>();
            let ch_list_message =
                format!("タグ {tag} に登録されたチャンネルは以下です。 {ch_name_list:#?}");
            send_message(ch_list_message).await?;
        }
        _ => {
            let undefined_message = "このコマンド引数は未定義です。".to_string();
            send_message(undefined_message).await?;
        }
    };

    Ok(SlackCommandEventResponse::new(SlackMessageContent::new()))
}
