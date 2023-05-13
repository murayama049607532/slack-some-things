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

static PATH_CH_LIST_FOLDER: &str = "ch_list_folder.json";

#[derive(Serialize, Deserialize, Debug, Clone, Builder, PartialEq, Eq)]
pub struct FolderSettings {
    ch_list: HashSet<SlackChannelId>,
    // #[default = "false"]
    // pub reaction: bool,
    #[default = "false"]
    bot: bool,
    private_user: Option<SlackUserId>,
}
impl FolderSettings {
    pub fn is_target(&self, target: &SlackChannelId) -> bool {
        self.ch_list.contains(target)
    }
    pub fn get_bot(&self) -> bool {
        self.bot
    }
    pub fn get_private_user(&self) -> Option<&SlackUserId> {
        self.private_user.as_ref()
    }
    // private_user:None => true
    pub fn has_auth(&self, user: &SlackUserId) -> bool {
        self.private_user.as_ref().map_or(true, |id| id == user)
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelListFolders(HashMap<String, FolderSettings>);

impl ChannelListFolders {
    //priv において、名前が衝突した場合の対処が必要
    fn add_channel_list(
        &mut self,
        folder: &str,
        channel: SlackChannelId,
        private_user: Option<SlackUserId>,
    ) -> &Self {
        self.0
            .entry(folder.to_string())
            .or_insert_with(|| FolderSettings::new(HashSet::new()).opt_private_user(private_user))
            .ch_list
            .insert(channel);
        self
    }
    fn delete_channel_list(&mut self, folder: &str, channel: &SlackChannelId) -> &Self {
        if let Some(folder_settings) = self.0.get_mut(folder) {
            folder_settings.ch_list.remove(channel);
            if folder_settings.ch_list.is_empty() {
                self.0.remove(folder);
            }
        }
        self
    }
    pub fn get_forder_settings(&self, folder: &str) -> Option<&FolderSettings> {
        let settings = self.0.get(folder);
        settings
    }
    fn get_channel_list(&self, folder: &str) -> Option<HashSet<SlackChannelId>> {
        let ch_list = self
            .0
            .get(folder)
            .map(|folder_setting| folder_setting.ch_list.clone());
        ch_list
    }
    fn retrieve_bot(&mut self, folder: &str, do_retrieve_bot: bool) -> &Self {
        self.0
            .entry(folder.to_string())
            .or_insert_with(|| FolderSettings::new(HashSet::new()))
            .bot = do_retrieve_bot;
        self
    }
    fn get_folder_private_user(&self, folder: &str) -> Option<&SlackUserId> {
        let priv_user = self
            .0
            .get(folder)
            .and_then(FolderSettings::get_private_user);
        priv_user
    }
    fn has_auth_for_folder(&self, user: &SlackUserId, folder: &str) -> bool {
        let has_auth = self.0.get(folder).map_or(true, |s| s.has_auth(user));
        has_auth
    }
}

pub async fn load_ch_list_folders_json() -> anyhow::Result<ChannelListFolders> {
    let path_ch_list = Path::new(PATH_CH_LIST_FOLDER);
    let mut ch_list_folders_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path_ch_list)
        .await?;

    let mut ch_list_folders_json = String::new();
    ch_list_folders_file
        .read_to_string(&mut ch_list_folders_json)
        .await?;

    let ch_list_folders: ChannelListFolders =
        serde_json::from_str(&ch_list_folders_json).unwrap_or(ChannelListFolders(HashMap::new()));

    Ok(ch_list_folders)
}

pub async fn add_channel_list(
    folder: &str,
    channel: SlackChannelId,
    private_user: Option<SlackUserId>,
) -> anyhow::Result<()> {
    let path_ch_list = Path::new(PATH_CH_LIST_FOLDER);
    let mut ch_list_folders: ChannelListFolders = load_ch_list_folders_json().await?;

    let ch_list_folders = ch_list_folders.add_channel_list(folder, channel, private_user);

    let new_content = serde_json::to_string_pretty(ch_list_folders)?;
    utils::update_json(path_ch_list, new_content).await?;

    Ok(())
}
pub async fn delete_channel_list(
    folder: &str,
    channel: SlackChannelId,
    user: SlackUserId,
) -> anyhow::Result<()> {
    let path_ch_list = Path::new(PATH_CH_LIST_FOLDER);
    let mut ch_list_folders: ChannelListFolders = load_ch_list_folders_json().await?;
    let has_auth = ch_list_folders.has_auth_for_folder(&user, folder);
    if !has_auth {
        Err(anyhow::anyhow!("you don't have auth for manage this tag."))?;
    }

    let ch_list_folders = ch_list_folders.delete_channel_list(folder, &channel);

    let new_content = serde_json::to_string_pretty(ch_list_folders)?;
    utils::update_json(path_ch_list, new_content).await?;

    Ok(())
}
pub async fn get_channel_list(
    folder_name: &str,
    user: SlackUserId,
) -> anyhow::Result<HashSet<SlackChannelId>> {
    let ch_list_folders: ChannelListFolders = load_ch_list_folders_json().await?;
    let has_auth = ch_list_folders.has_auth_for_folder(&user, folder_name);
    if !has_auth {
        Err(anyhow::anyhow!("you don't have auth for manage this tag."))?;
    }

    let ch_list = &ch_list_folders
        .get_channel_list(folder_name)
        .context("the tag does not exist")?;
    Ok(ch_list.clone())
}
pub async fn change_retrieve_bot(
    folder_name: &str,
    do_retrieve_bot: bool,
    user: SlackUserId,
) -> anyhow::Result<()> {
    let path_ch_list = Path::new(PATH_CH_LIST_FOLDER);
    let mut ch_list_folders: ChannelListFolders = load_ch_list_folders_json().await?;

    let has_auth = ch_list_folders.has_auth_for_folder(&user, folder_name);
    if !has_auth {
        Err(anyhow::anyhow!("you don't have auth for manage this tag."))?;
    }

    let ch_list_folders = ch_list_folders.retrieve_bot(folder_name, do_retrieve_bot);
    let new_content = serde_json::to_string_pretty(ch_list_folders)?;

    utils::update_json(path_ch_list, new_content).await?;
    Ok(())
}
pub async fn get_tag_list() -> anyhow::Result<Vec<String>> {
    let ch_list_folders: ChannelListFolders = load_ch_list_folders_json().await?;
    let tags = ch_list_folders.0.into_keys().collect::<Vec<_>>();
    Ok(tags)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_add_channel_list() {
        let test_ch = SlackChannelId::new("C01234567".to_string());
        add_channel_list("test", test_ch.clone(), None)
            .await
            .unwrap();

        let channel_folder = load_ch_list_folders_json().await.unwrap();
        let has_channel = channel_folder
            .get_channel_list("test")
            .unwrap()
            .contains(&test_ch);
        assert!(has_channel);
    }
    #[tokio::test]
    async fn delete_channel_list_test() {
        let folder_list = load_ch_list_folders_json().await.unwrap();
        let test_ch = SlackChannelId::new("C987654321".to_string());
        let _ = add_channel_list("test", test_ch.clone(), None).await;

        let _ = delete_channel_list("test", test_ch, SlackUserId::new("test".to_string())).await;
        let add_delete_folder_list = load_ch_list_folders_json().await.unwrap();

        assert_eq!(folder_list.0, add_delete_folder_list.0);
    }
}
