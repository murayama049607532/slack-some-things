use std::{str::SplitWhitespace, sync::Arc};

use anyhow::Ok;
use slack_morphism::{prelude::SlackHyperClient, SlackChannelId, SlackUserId};

use crate::post_message::MessagePoster;

const HELP_TEXT:&str = "Channel Bugyo は、カスタマイズ可能なチャンネルメッセージ集約を行うためのSlack用アプリケーションです。
このアプリでは、収集したいチャンネルを管理するためにタグを使用します。タグにより、特定のトピックやプロジェクトごとに関連するチャンネルをまとめることができます。
タグには、ユーザーのみがアクセスできる「ユーザータグ」と、誰でもアクセスできる「パブリックタグ」という2つの種類があります。
「add」コマンドを使用すると、特定のタグにチャンネルを登録できます。例えば、add --publicというオプションを追加すると、パブリックなタグの管理が可能です。
使用可能なコマンドとして以下が存在し、 `/channel_bugyo help add` のように呼び出すことで、コマンドごとのヘルプを閲覧可能です。
使用可能なコマンド： `add, delete, retrieve_bot, ch_list, tag_list, set, unset, create_channel, target_list`";

const ADD_TEXT:&str = "指定したタグにチャンネルを追加します。デフォルトではプライベートタグとして、登録したユーザのみがアクセス可能です.。
`/channel_bugyo add [tag] [#channel_1] [#channel_2] [#channel_3] ...`
`/channel_bugyo add --public [tag] [#channel_1] [#channel_2] [#channel_3] ...`";

const DELETE_TEXT: &str = "指定したタグからチャンネルを削除します。
`/channel_bugyo delete [tag] [#channel_1] [#channel_2] [#channel_3] ...`
`/channel_bugyo delete --public [tag] [#channel_1] [#channel_2] [#channel_3] ...`";

const RETBOT_TEXT:&str = "指定したタグがボットによるメッセージを収集するかを設定します。（初期値は false)
第二引数が true であれば、ボットメッセージを収集するようになり、false であれば、ボットメッセージを無視します。
`/channel_bugyo retrieve_bot [tag] [bool]`
`/channel_bugyo retrieve_bot --public [tag] [bool]`";

const CH_LS_TEXT: &str = "指定したタグの収集対象チャンネルの一覧を表示します。
`/channel_bugyo ch_list [tag]`
`/channel_bugyo ch_list --public [tag]`";

const TAG_LS_TEXT: &str = "存在するタグの一覧を表示します。
`/channel_bugyo tag_list`";
const SET_TEXT:&str = "Channel Bugyo が追加されているチャンネルにおいて使用することで、そのチャンネルに、指定したタグで収集対象となっているチャンネルのメッセージを収集します。
`/channel_bugyo set [tag_1] [tag_2] [tag_3] ...`
`/channel_bugyo set --public [tag_1] [tag_2] [tag_3] ...`";

const UNSET_TEXT: &str = "set されているタグを収集対象から外します。
`/channel_bugyo unset [tag_1] [tag_2] [tag_3] ...`
`/channel_bugyo unset --public [tag_1] [tag_2] [tag_3] ...`";

const CREATE_TEXT: &str = "指定したタグを収集対象とする新たなプライベートチャンネルを作成します。
`/channel_bugyo create_channel [new_channel_name] [tag_1] [tag_2] [tag_3] ...`
`/channel_bugyo create_channel --public [new_channel_name] [tag_1] [tag_2] [tag_3] ...`";

const TARGET_LS_TEXT: &str = "現在チャンネルが収集対象としているタグの一覧を表示します。
`/channel_bugyo target_list`";

const UNDEFINED_TEXT: &str = "このコマンドは未定義です。";

pub async fn help(
    cli: Arc<SlackHyperClient>,
    channel_id_command: SlackChannelId,
    user_id_command: SlackUserId,
    mut args_iter: SplitWhitespace<'_>,
) -> anyhow::Result<()> {
    let first = args_iter.next().unwrap_or("help");
    let help_text = choose_text(first);
    let _ = MessagePoster::new(channel_id_command, help_text, cli)
        .post_ephemeral(user_id_command)
        .await?;
    Ok(())
}
fn choose_text(arg: &str) -> String {
    match arg {
        "help" => HELP_TEXT,
        "add" => ADD_TEXT,
        "delete" => DELETE_TEXT,
        "retrieve_bot" => RETBOT_TEXT,
        "ch_list" => CH_LS_TEXT,
        "tag_list" => TAG_LS_TEXT,
        "set" => SET_TEXT,
        "unset" => UNSET_TEXT,
        "create_channel" => CREATE_TEXT,
        "target_list" => TARGET_LS_TEXT,
        _ => UNDEFINED_TEXT,
    }
    .to_string()
}
