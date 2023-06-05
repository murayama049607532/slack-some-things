use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use slack_morphism::{
    prelude::{
        SlackApiBotsInfoRequest, SlackApiBotsInfoResponse, SlackApiUsersProfileGetRequest,
        SlackApiUsersProfileGetResponse, SlackHyperClient,
    },
    SlackApiTokenType, SlackBotId, SlackMessageSender, SlackUserId,
};

use crate::utils::get_token;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SenderProfile {
    pub icon_url: url::Url,
    pub name: String,
}

pub async fn fetch_profile(
    cli: Arc<SlackHyperClient>,
    sender: SlackMessageSender,
) -> anyhow::Result<SenderProfile> {
    match sender.user {
        Some(user_id) => {
            let user_profile = fetch_user_profile(cli, user_id).await?;
            let user_icon = user_profile.get_icon_url()?;
            let user_name = user_profile.get_display_name()?;
            Ok(SenderProfile {
                icon_url: user_icon,
                name: user_name,
            })
        }
        None => match sender.bot_id {
            Some(bot_id) => {
                let bot_profile = fetch_bot_info(cli, bot_id).await?;
                let bot_icon = bot_profile.get_icon_url()?;
                let bot_name = bot_profile.get_display_name()?;
                Ok(SenderProfile {
                    icon_url: bot_icon,
                    name: bot_name,
                })
            }
            None => Err(anyhow::anyhow!("cannot identify sender")),
        },
    }
}

pub async fn fetch_user_profile(
    cli: Arc<SlackHyperClient>,
    user_id: SlackUserId,
) -> anyhow::Result<SlackApiUsersProfileGetResponse> {
    let token = get_token(&SlackApiTokenType::Bot)?;
    let session = cli.open_session(&token);
    let user_profile_req = SlackApiUsersProfileGetRequest::new().with_user(user_id);
    let res = session
        .users_profile_get(&user_profile_req)
        .await
        .context("failed to get user's icon")?;
    Ok(res)
}
pub async fn fetch_bot_info(
    cli: Arc<SlackHyperClient>,
    bot_id: SlackBotId,
) -> anyhow::Result<SlackApiBotsInfoResponse> {
    let token = get_token(&SlackApiTokenType::Bot)?;
    let session = cli.open_session(&token);
    let bot_info_req = SlackApiBotsInfoRequest::new().with_bot(bot_id.0);
    let res = session
        .bots_info(&bot_info_req)
        .await
        .context("failed to get bot's info")?;
    Ok(res)
}

pub trait GetterProfile {
    fn get_icon_url(&self) -> anyhow::Result<url::Url>;
    fn get_display_name(&self) -> anyhow::Result<String>;
}
impl GetterProfile for SlackApiUsersProfileGetResponse {
    fn get_icon_url(&self) -> anyhow::Result<url::Url> {
        let icon_str = &self
            .profile
            .icon
            .clone()
            .context("no icon")?
            .image_original
            .context("failed to get icon")?;

        let icon_url = url::Url::parse(icon_str)?;
        Ok(icon_url)
    }

    fn get_display_name(&self) -> anyhow::Result<String> {
        let real_name = self.profile.real_name.clone().unwrap_or(String::new());
        let display_name = self
            .profile
            .display_name
            .clone()
            .map_or(String::new(), |display| {
                if display.is_empty() {
                    real_name
                } else {
                    display
                }
            });
        Ok(display_name)
    }
}
impl GetterProfile for SlackApiBotsInfoResponse {
    fn get_icon_url(&self) -> anyhow::Result<url::Url> {
        let icons = self
            .bot
            .icons
            .clone()
            .context("failed to get icon images:bot")?;
        let icon = icons.resolutions.get(0).cloned().unwrap_or((
            512,
            "https://avatars.slack-edge.com/2023-03-18/4975228596980_b7f6572d76d9104bbc72_512.png"
                .to_string(),
        ));
        let icon_str = &icon.1;
        let icon_url = url::Url::parse(icon_str)?;
        Ok(icon_url)
    }
    fn get_display_name(&self) -> anyhow::Result<String> {
        let bot_name = &self.bot.name;
        Ok(bot_name.to_string())
    }
}
