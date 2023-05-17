use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Context;
use slack_morphism::{
    prelude::{
        SlackClientEventsUserState, SlackCommandEvent, SlackCommandEventResponse, SlackHyperClient,
    },
    SlackMessageContent,
};

use crate::{
    commands::{self, create_channel, operate, set_target_tags},
    post_message::MessagePoster,
};

// retrurn response to slack early, to avoid timeout error
pub async fn spawned_command_handler(
    event: SlackCommandEvent,
    cli: Arc<SlackHyperClient>,
    state: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    tokio::spawn(async move {
        let _ = handler_catch_error(event, cli, state).await;
    });
    Ok(SlackCommandEventResponse::new(SlackMessageContent::new()))
}

pub async fn handler_catch_error(
    event: SlackCommandEvent,
    cli: Arc<SlackHyperClient>,
    state: SlackClientEventsUserState,
) -> anyhow::Result<()> {
    match command_event_handler(event.clone(), cli.clone(), state).await {
        Ok(_) => Ok(()),
        Err(err) => {
            let err_message = format!("{err:#?}");
            MessagePoster::new(event.channel_id, err_message, cli)
                .post_ephemeral(event.user_id)
                .await?;
            Ok(())
        }
    }
}

pub async fn command_event_handler(
    event: SlackCommandEvent,
    cli: Arc<SlackHyperClient>,
    _state: SlackClientEventsUserState,
) -> anyhow::Result<()> {
    println!("{event:#?}");
    let channel_id_command = event.channel_id.clone();
    let user_id_command = event.user_id;

    let full = event.text.clone().unwrap_or(String::new());
    let mut args_iter = full.split_whitespace();
    let first_arg = args_iter.next().context("argument error")?;

    match first_arg {
        "add" => operate::add_command(cli, channel_id_command, user_id_command, args_iter).await?,
        "delete" => {
            operate::delete_command(cli, channel_id_command, user_id_command, args_iter).await?;
        }
        "set" => {
            set_target_tags::set_command(cli, channel_id_command, user_id_command, args_iter)
                .await?;
        }
        "unset" => {
            set_target_tags::unset_command(cli, channel_id_command, user_id_command, args_iter)
                .await?;
        }
        "create_channel" => {
            create_channel::create_command(
                cli.clone(),
                channel_id_command,
                user_id_command.clone(),
                args_iter,
            )
            .await?;
        }
        "retrieve_bot" => {
            operate::retreieve_bot_command(cli, channel_id_command, user_id_command, args_iter)
                .await?;
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

    Ok(())
}
