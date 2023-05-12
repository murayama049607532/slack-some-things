#![warn(clippy::pedantic)]
mod command_event_handler;
mod create_channel;
mod dist_target_map;
mod process_message;
mod push_event_handler;
mod set_target_tags;
mod slack_sender;
mod utils;

use slack_morphism::prelude::*;
use std::sync::Arc;

async fn socket_mode_process() -> anyhow::Result<()> {
    let app_token = utils::get_token(&SlackApiTokenType::App)?;
    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));
    let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
        .with_push_events(push_event_handler::push_event_handler)
        .with_command_events(command_event_handler::command_event_handler);
    let listner_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            .with_error_handler(push_event_handler::error_handler),
    );
    let socket_mode_listner = SlackClientSocketModeListener::new(
        &SlackClientSocketModeConfig::new(),
        listner_environment.clone(),
        socket_mode_callbacks,
    );

    socket_mode_listner.listen_for(&app_token).await?;
    socket_mode_listner.serve().await;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    socket_mode_process().await?;

    Ok(())
}
