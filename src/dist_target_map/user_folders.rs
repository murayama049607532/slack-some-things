use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slack_morphism::SlackUserId;

use super::channel_list_folder::ChannelListFolder;

pub const PUBLIC_TAGS: &str = "***public***";

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct UserFolders(HashMap<String, ChannelListFolder>);

impl UserFolders {
    pub fn user_ch_list_folders(&self, user: &SlackUserId) -> ChannelListFolder {
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
    pub fn public_ch_list_folders(&self) -> ChannelListFolder {
        let public = SlackUserId::new(PUBLIC_TAGS.to_string());
        self.user_ch_list_folders(&public)
    }

    pub fn is_valid(&self, user: &SlackUserId, tag: &str) -> bool {
        self.user_ch_list_folders(user).has_tag(tag)
    }
    pub fn user_tag_list(&self, user: &SlackUserId) -> Vec<String> {
        self.user_ch_list_folders(user).get_tag_list()
    }
    pub fn public_tag_list(&self) -> Vec<String> {
        self.public_ch_list_folders().get_tag_list()
    }
}
