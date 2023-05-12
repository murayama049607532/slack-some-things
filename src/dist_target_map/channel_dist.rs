use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use slack_morphism::SlackChannelId;
use tokio::{fs::OpenOptions, io::AsyncReadExt};

use crate::utils;

const PATH_CH_DISTS_FOLDER: &str = "ch_dists.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelDists(HashMap<SlackChannelId, HashSet<String>>);

impl ChannelDists {
    fn add_tag_to_dist(&mut self, dist: SlackChannelId, tag: &str) -> &Self {
        self.0
            .entry(dist)
            .or_insert_with(HashSet::new)
            .insert(tag.to_string());
        self
    }
}
async fn load_ch_dists_json() -> anyhow::Result<ChannelDists> {
    let path_ch_dists = Path::new(PATH_CH_DISTS_FOLDER);
    let mut ch_dists_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path_ch_dists)
        .await?;

    let mut ch_dists_json = String::new();
    ch_dists_file.read_to_string(&mut ch_dists_json).await?;

    let ch_dists = serde_json::from_str(&ch_dists_json).unwrap_or(ChannelDists(HashMap::new()));

    Ok(ch_dists)
}

pub async fn add_dists_json(dist: SlackChannelId, tag: &str) -> anyhow::Result<()> {
    let path_ch_dists = Path::new(PATH_CH_DISTS_FOLDER);
    let mut ch_dists = load_ch_dists_json().await?;

    let ch_dists_new = ch_dists.add_tag_to_dist(dist, tag);
    let new_content = serde_json::to_string_pretty(ch_dists_new)?;
    utils::update_json(path_ch_dists, new_content).await?;
    Ok(())
}
pub async fn get_channel_tags(dist: SlackChannelId) -> anyhow::Result<HashSet<String>> {
    let ch_dists = load_ch_dists_json().await?;
    let tags = ch_dists
        .0
        .get(&dist)
        .context("the channel does not have tag")?;
    Ok(tags.clone())
}
pub async fn get_dists_list() -> anyhow::Result<Vec<SlackChannelId>> {
    let ch_dists = load_ch_dists_json().await?;
    let dists_list = ch_dists.0.keys().cloned().collect::<Vec<_>>();
    Ok(dists_list)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_dists() {
        let test_ch = SlackChannelId::new("C012345678".to_string());
        add_dists_json(test_ch.clone(), "poi").await.unwrap();

        let dists = load_ch_dists_json().await.unwrap();

        let has_channel = dists.0.get(&test_ch).unwrap().contains("poi");
        assert!(has_channel);
    }
}
