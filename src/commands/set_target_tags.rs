use futures::StreamExt;

use slack_morphism::{SlackChannelId, SlackUserId};

use crate::dist_target_map::{self, channel_dist};

pub async fn set_targets(
    channel_id: &SlackChannelId,
    user_command: SlackUserId,
    tags: &[String],
) -> anyhow::Result<()> {
    let user_folders = dist_target_map::operate_folder::load_user_folders_json().await?;
    let auth_tags_iter = tags
        .iter()
        .filter(|tag| user_folders.is_valid_for_user(&user_command, tag));

    let tags_stream = futures::stream::iter(auth_tags_iter);

    tags_stream
        .for_each(|tag| async {
            channel_dist::add_dists_json(channel_id.clone(), user_command.clone(), tag)
                .await
                .unwrap_or(());
        })
        .await;
    Ok(())
}
