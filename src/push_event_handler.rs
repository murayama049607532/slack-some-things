use std::sync::Arc;

use anyhow::Context;
use slack_morphism::{
    prelude::{
        events::SlackEventCallbackBody::*, SlackApiChatPostMessageRequest, SlackClientEventsUserState, SlackHyperClient,
        SlackPushEventCallback,
    },
    SlackMessageContent,
};
use tokio_stream::StreamExt;

use crate::{
    dist_target_map::get_all_map,
    process_message::{self, sender_profile::fetch_profile},
    send_message::{self, SlackApiMessageRequest, SlackApiMessageResponse},
};

pub async fn push_event_handler(
    event: SlackPushEventCallback,
    cli: Arc<SlackHyperClient>,
    _state: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //println!("{event:#?}");
    match event.event {
        Message(msg_event) => {
            let channel_id_from = msg_event
                .clone()
                .origin
                .channel
                .context("cannot get channel id")?;
            let dist_target_map = get_all_map().await?;
            let is_target = dist_target_map.is_target(msg_event.clone())?;
            if !is_target {
                return Ok(());
            }

            let dists = dist_target_map.target_to_dists(&channel_id_from);
            let sender = msg_event.clone().sender;

            let sender_profile = fetch_profile(cli.clone(), sender).await?;

            let message_reqs = dists
                .iter()
                .map(|dist| {
                    process_message::message_event_to_req(
                        msg_event.clone(),
                        dist.clone(),
                        sender_profile.clone(),
                    )
                    .unwrap_or_else(|err| {
                        let err_message = err.to_string();
                        SlackApiChatPostMessageRequest::new(
                            dist.clone(),
                            SlackMessageContent::new().with_text(err_message),
                        )
                    })
                })
                .collect::<Vec<_>>();

            let req_stream = futures::stream::iter(message_reqs);
            let ress = req_stream
                .map(|msg_req| {
                    let cli_clone = Arc::clone(&cli);
                    async move {
                        let res = send_message::send_message(
                            cli_clone,
                            SlackApiMessageRequest::PostMessage(msg_req.clone()),
                        )
                        .await?;
                        anyhow::Ok::<SlackApiMessageResponse>(res)
                    }
                })
                .then(|s| async { s.await })
                .collect::<Vec<_>>()
                .await;
            println!("{ress:#?}");
        }
        MemberJoinedChannel(_join_event) => {}
        MemberLeftChannel(_left_event) => {}
        _ => {}
    };

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> http::StatusCode {
    println!("err:{err:#?}");
    http::StatusCode::OK
}
