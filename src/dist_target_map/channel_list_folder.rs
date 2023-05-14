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

pub const PUBLIC_TAGS: &str = "***public***";

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq, Eq)]
pub struct FolderSettings {
    ch_list: HashSet<SlackChannelId>,
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
pub struct ChannelListFolder(HashMap<String, FolderSettings>);

impl ChannelListFolder {
    pub fn add_channel_list(&mut self, folder_tag: String, channel: SlackChannelId) -> &Self {
        self.0
            .entry(folder_tag)
            .or_insert_with(|| FolderSettings::new(HashSet::new()))
            .ch_list
            .insert(channel);
        self
    }
    pub fn delete_channel_list(&mut self, folder_tag: &str, channel: &SlackChannelId) -> &Self {
        if let Some(folder_settings) = self.0.get_mut(folder_tag) {
            folder_settings.ch_list.remove(channel);
            if folder_settings.ch_list.is_empty() {
                self.0.remove(folder_tag);
            }
        }
        self
    }
    pub fn get_folder_settings(&self, tag: &str) -> Option<FolderSettings> {
        let settings = self.0.get(tag).cloned();
        settings
    }
    pub fn retrieve_bot(&mut self, folder_tag: String, do_retrieve_bot: bool) -> &Self {
        self.0
            .entry(folder_tag)
            .or_insert_with(|| FolderSettings::new(HashSet::new()))
            .bot = do_retrieve_bot;
        self
    }
    pub fn get_channel_list(&self, folder_tag: &str) -> Option<HashSet<SlackChannelId>> {
        let ch_list = self
            .0
            .get(folder_tag)
            .map(|folder_setting| folder_setting.ch_list.clone());
        ch_list
    }
    pub fn get_tag_list(&self) -> Vec<String> {
        let tag_list = self.0.keys().cloned().collect::<Vec<_>>();
        tag_list
    }
    pub fn has_tag(&self, tag: &str) -> bool {
        self.0.contains_key(tag)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct UserFolders(HashMap<String, ChannelListFolder>);

impl UserFolders {
    pub fn get_user_ch_list_folders(&self, user: &SlackUserId) -> ChannelListFolder {
        let folder = self.0.get(&user.to_string()).cloned().unwrap_or_default();
        folder
    }
    pub fn mut_user_ch_list_folders(&mut self, user: &SlackUserId) -> &mut ChannelListFolder {
        let folder = self
            .0
            .entry(user.to_string())
            .or_insert(ChannelListFolder::default());
        folder
    }
    pub fn is_valid_for_user(&self, user: &SlackUserId, tag: &str) -> bool {
        let valid_for_user = self.get_user_ch_list_folders(user).has_tag(tag);
        let valid_for_public = self
            .get_user_ch_list_folders(&SlackUserId::new(PUBLIC_TAGS.to_string()))
            .has_tag(tag);
        valid_for_public || valid_for_user
    }
}
