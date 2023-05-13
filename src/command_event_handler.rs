use std::sync::Arc;

use anyhow::Context;
use slack_morphism::{
    prelude::{
        SlackApiChatPostEphemeralRequest, SlackApiChatPostMessageRequest,
        SlackClientEventsUserState, SlackCommandEvent, SlackCommandEventResponse, SlackHyperClient,
    },
    SlackMessageContent,
};

use crate::{
    commands,
    post_message::{self, SlackApiMessageRequest},
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

    match first_arg {
        "add" => commands::add_command(cli, channel_id_command, args_iter).await?,
        "delete" => commands::delete_command(cli, channel_id_command, args_iter).await?,
        "set" => commands::set_command(cli, channel_id_command, args_iter).await?,
        "create_channel" => {
            commands::create_command(
                cli.clone(),
                channel_id_command,
                user_id_command.clone(),
                args_iter,
            )
            .await?;
        }
        "retrieve_bot" => {
            commands::retreieve_bot_command(cli, channel_id_command, args_iter).await?;
        }

        "tag_list" => {
            commands::tag_list_command(cli, channel_id_command, user_id_command).await?;
        }
        "ch_list" => {
            commands::ch_list_command(cli, channel_id_command, user_id_command, args_iter).await?;
        }
        _ => {
            commands::undefined_command(cli, channel_id_command, user_id_command).await?;
        }
    };

    Ok(SlackCommandEventResponse::new(SlackMessageContent::new()))
}
