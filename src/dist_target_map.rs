pub mod channel_dist;
pub mod channel_list_folder;

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use futures::StreamExt;
use slack_morphism::{prelude::SlackMessageEvent, SlackChannelId};

use crate::utils;

use channel_list_folder::FolderSettings;

pub async fn get_target_folder_list(dist: SlackChannelId) -> anyhow::Result<Vec<FolderSettings>> {
    let tags = channel_dist::get_channel_tags(dist).await?;
    let ch_list_folders = channel_list_folder::load_ch_list_folders_json().await?;

    let target_folders = tags
        .into_iter()
        .map(|s| {
            let folder_settings = ch_list_folders
                .get_forder_settings(&s)
                .cloned()
                .unwrap_or(FolderSettings::new(HashSet::new()));
            folder_settings
        })
        .collect();

    Ok(target_folders)
}

pub struct DistTargetMap(HashMap<SlackChannelId, Vec<FolderSettings>>);

impl DistTargetMap {
    pub fn target_to_dists(&self, target: &SlackChannelId) -> Vec<SlackChannelId> {
        let dists = self
            .0
            .iter()
            .filter(|(_, targets)| targets.iter().any(|setting| setting.is_target(target)))
            .map(|(dist, _)| dist.clone())
            .collect();
        dists
    }
    pub fn is_target(&self, msg_event: SlackMessageEvent) -> anyhow::Result<bool> {
        let all_settings = self.0.values().flatten().cloned().collect::<Vec<_>>();
        let self_bot = utils::get_self_bot_id()?;
        let is_bot = msg_event.sender.bot_id.is_some();
        let is_self = msg_event
            .sender
            .bot_id
            .map_or(false, |bot_id| bot_id == self_bot);

        let channel_from = msg_event
            .origin
            .channel
            .context("failed to get channel id")?;

        let is_target = all_settings.iter().any(|settings| {
            let is_contain = settings.is_target(&channel_from);
            let retrieve_bot = settings.get_bot();
            (!is_bot || retrieve_bot) && is_contain && !is_self
        });
        Ok(is_target)
    }
}

pub async fn get_all_map() -> anyhow::Result<DistTargetMap> {
    let dists = channel_dist::get_dists_list().await?;
    let dists_stream = futures::stream::iter(dists);
    let dist_target_map = dists_stream
        .map(|s| async {
            let targets = get_target_folder_list(s.clone())
                .await
                .unwrap_or(Vec::new());
            (s, targets)
        })
        .then(|s| async { s.await })
        .collect::<HashMap<_, _>>()
        .await;
    Ok(DistTargetMap(dist_target_map))
}
