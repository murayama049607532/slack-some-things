use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{Context, Ok};
use rsb_derive::Builder;
use serde::{Deserialize, Serialize};
use slack_morphism::{SlackChannelId, SlackUserId};

use tokio::{fs::OpenOptions, io::AsyncReadExt};

use crate::utils;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Builder)]
pub struct FolderTag {
    pub tag: String,
    pub owner_id: Option<SlackUserId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq, Eq)]
pub struct FolderSettings {
    ch_list: HashSet<SlackChannelId>,
    // #[default = "false"]
    // pub reaction: bool,
    #[default = "false"]
    bot: bool,
}
impl FolderSettings {
    pub fn is_target(&self, target: &SlackChannelId) -> bool {
        self.ch_list.contains(target)
    }
    pub fn get_bot(&self) -> bool {
        self.bot
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct ChannelListFolders(HashMap<FolderTag, FolderSettings>);

impl ChannelListFolders {
    pub fn add_channel_list(&mut self, folder_tag: FolderTag, channel: SlackChannelId) -> &Self {
        let folder_tag = self
            .0
            .entry(folder_tag)
            .or_insert_with(|| FolderSettings::new(HashSet::new()))
            .ch_list
            .insert(channel);
        self
    }
    pub fn delete_channel_list(
        &mut self,
        folder_tag: FolderTag,
        channel: &SlackChannelId,
    ) -> &Self {
        if let Some(folder_settings) = self.0.get_mut(&folder_tag) {
            folder_settings.ch_list.remove(channel);
            if folder_settings.ch_list.is_empty() {
                self.0.remove(&folder_tag);
            }
        }
        self
    }
    pub fn get_forder_settings(&self, tag: &str) -> Vec<FolderSettings> {
        let settings = self
            .0
            .clone()
            .into_iter()
            .filter(|(k, _)| &k.tag == tag)
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        settings
    }
    pub fn retrieve_bot(&mut self, folder_tag: FolderTag, do_retrieve_bot: bool) -> &Self {
        self.0
            .entry(folder_tag)
            .or_insert_with(|| FolderSettings::new(HashSet::new()))
            .bot = do_retrieve_bot;
        self
    }
    pub fn get_channel_list(&self, folder_tag: FolderTag) -> Option<HashSet<SlackChannelId>> {
        let ch_list = self
            .0
            .get(&folder_tag)
            .map(|folder_setting| folder_setting.ch_list.clone());
        ch_list
    }
    pub fn get_tag_list(&self, user_id: &SlackUserId) -> Vec<String> {
        let user_tag_list = self
            .0
            .iter()
            .filter(|(k, _)| k.owner_id.is_none() || k.owner_id == Some(user_id.clone()))
            .map(|(tag, _)| tag.tag.clone())
            .collect::<Vec<_>>();
        user_tag_list
    }
}
