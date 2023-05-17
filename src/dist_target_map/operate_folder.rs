use std::{collections::HashSet, path::Path};

use anyhow::Context;
use slack_morphism::{SlackChannelId, SlackUserId};
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use crate::utils;

use super::{channel_list_folder::ChannelListFolder, user_folders::UserFolders};

#[derive(Debug, Clone)]
pub enum FolderOperation {
    Add,
    Delete,
    RetrieveBot,
}

const PATH_CH_LIST_FOLDER: &str = "ch_list_folder.json";

pub async fn load_user_folders_json_from_path(path_ch_list: &Path) -> anyhow::Result<UserFolders> {
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

    let ch_list_folders: UserFolders =
        serde_json::from_str(&ch_list_folders_json).unwrap_or_default();

    Ok(ch_list_folders)
}

pub async fn load_user_folders_json() -> anyhow::Result<UserFolders> {
    let path_ch_list = Path::new(PATH_CH_LIST_FOLDER);
    load_user_folders_json_from_path(path_ch_list).await
}

pub async fn operate_channel_list(
    folder: String,
    channel: SlackChannelId,
    user: SlackUserId,
    operation: FolderOperation,
    retrieve_bot: Option<bool>,
) -> anyhow::Result<()> {
    let path_ch_list = Path::new(PATH_CH_LIST_FOLDER);
    operate_channel_list_from_path(folder, channel, user, operation, retrieve_bot, path_ch_list)
        .await?;
    Ok(())
}
pub async fn operate_channel_list_from_path(
    folder: String,
    channel: SlackChannelId,
    user: SlackUserId,
    operation: FolderOperation,
    retrieve_bot: Option<bool>,
    path_ch_list: &Path,
) -> anyhow::Result<()> {
    let mut user_folders = load_user_folders_json_from_path(path_ch_list).await?;
    let ch_list_folders = user_folders.mut_user_ch_list_folders(&user);

    let _ = match operation {
        FolderOperation::Add => ch_list_folders.add_channel_list(folder, channel),
        FolderOperation::Delete => ch_list_folders.delete_channel_list(&folder, &channel),
        FolderOperation::RetrieveBot => {
            let retrieve_bot_unwrap = retrieve_bot.context("retrieve bot must be bool")?;
            ch_list_folders.retrieve_bot(folder, retrieve_bot_unwrap)
        }
    };

    let new_content = serde_json::to_string_pretty(&user_folders)?;
    utils::update_json(path_ch_list, new_content).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static PATH_CH_LIST_FOLDER_TEST: &str = "ch_list_folder_test.json";

    async fn load_ch_list_folders_json_test(
        user: SlackUserId,
    ) -> anyhow::Result<ChannelListFolder> {
        let path_test = Path::new(PATH_CH_LIST_FOLDER_TEST);
        let ch_list_folder = load_user_folders_json_from_path(path_test)
            .await?
            .user_ch_list_folders(&user);
        Ok(ch_list_folder)
    }
    async fn operate_channel_list_test(
        folder: String,
        channel: SlackChannelId,
        user: SlackUserId,
        operation: FolderOperation,
        retrieve_bot: Option<bool>,
    ) -> anyhow::Result<()> {
        let path_test = Path::new(PATH_CH_LIST_FOLDER_TEST);
        operate_channel_list_from_path(folder, channel, user, operation, retrieve_bot, path_test)
            .await?;
        Ok(())
    }

    #[ignore = "json"]
    #[tokio::test]
    async fn test_create_and_add_channel_list() {
        let test_ch = SlackChannelId::new("C01234567".to_string());
        let user_id = SlackUserId::new("U0123455".to_string());

        operate_channel_list_test(
            "test".to_string(),
            test_ch.clone(),
            user_id.clone(),
            FolderOperation::Add,
            None,
        )
        .await
        .unwrap();

        let channel_folder = load_ch_list_folders_json_test(user_id.clone())
            .await
            .unwrap();
        let folder_tag = "test".to_string();
        let has_channel = channel_folder
            .get_channel_list(&folder_tag)
            .unwrap()
            .contains(&test_ch);

        operate_channel_list_test(
            "test".to_string(),
            test_ch,
            user_id,
            FolderOperation::Delete,
            None,
        )
        .await
        .unwrap();

        assert!(has_channel);
    }

    #[tokio::test]
    async fn delete_channel_list_test() {
        let path_test = Path::new(PATH_CH_LIST_FOLDER_TEST);
        let user_id = SlackUserId::new("U01234557".to_string());
        let test_ch = SlackChannelId::new("C987654321".to_string());

        let ch_list_folder = load_user_folders_json_from_path(path_test)
            .await
            .unwrap()
            .user_ch_list_folders(&user_id);

        operate_channel_list_test(
            "test".to_string(),
            test_ch.clone(),
            user_id.clone(),
            FolderOperation::Add,
            None,
        )
        .await
        .unwrap();

        operate_channel_list_test(
            "test".to_string(),
            test_ch,
            user_id.clone(),
            FolderOperation::Delete,
            None,
        )
        .await
        .unwrap();
        let add_delete_folder_list = load_user_folders_json_from_path(path_test)
            .await
            .unwrap()
            .user_ch_list_folders(&user_id);

        assert_eq!(ch_list_folder, add_delete_folder_list);
    }
}
