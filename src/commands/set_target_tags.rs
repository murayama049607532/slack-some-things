use futures::StreamExt;

use slack_morphism::SlackChannelId;

use crate::dist_target_map::channel_dist;

pub async fn set_targets(channel_id: &SlackChannelId, tags: &[String]) -> anyhow::Result<()> {
    let tags_iter = tags.iter();
    let tags_stream = futures::stream::iter(tags_iter.clone());
    tags_stream
        .for_each(|tag| async {
            channel_dist::add_dists_json(channel_id.clone(), tag)
                .await
                .unwrap_or(());
        })
        .await;
    Ok(())
}
