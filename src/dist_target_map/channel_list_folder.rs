use std::collections::{HashMap, HashSet};

use rsb_derive::Builder;
use serde::{Deserialize, Serialize};
use slack_morphism::SlackChannelId;

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
