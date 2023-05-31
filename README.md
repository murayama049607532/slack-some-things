# Channel Bugyo

## 概要

Channel Bugyo（仮）は、カスタマイズ可能なチャンネルメッセージ集約を行うためのSlack用アプリケーションです。Channel Bugyo が追加されているチャンネル内で、以下に説明されている設定を行うことで使用可能になります。 \
現在はプレビュー版です。 \
アプリの権限設定については、manifest.yml を参照してください。

## 環境変数設定
.env ファイルを用意し、SlackApp用トークンと、ボットのIDを環境変数として設定してください。また、データベースURLを以下のように設定してください。（現在のところ、ハードコーディングされているため、別名での登録では動きません。） \

```
SLACK_APP_TOKEN=xapp-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
SLACK_BOT_TOKEN=xoxb-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
SLACK_USER_TOKEN=xoxp-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

SLACK_BOT_ID=BXXXXXXXX

DATABASE_URL=sqlite://sqlite.db
```
本プログラムでは、sqlx の query! 関数を使用しているため、コンパイル時にデータベースとテーブルが存在している必要があります。examples/sqlite_init.rs を実行することで、本プログラムで用いられるデータベースとテーブルの初期設定が行われます。

## 機能

### タグ

Channel Bugyo では、収集対象のチャンネルの集合を「タグ」として定義し、収集先チャンネルにタグをセットすることではじめてメッセージの収集を開始します。 \
タグには収集対象のチャンネルリスト、タグの所有者、ボットによるメッセージを収集対象とするかについて情報を持ちます。 \


#### add

指定したタグの channel list folder にチャンネルを追加します。デフォルトではプライベートタグとして、登録したユーザのみがアクセス可能です。 \
オプションとして、add の後に --public を追加することで、全ユーザがアクセス可能なパブリックタグを登録可能です。

`/channel_bugyo add [tag] [#channel_1] [#channel_2] [#channel_3] ...`

例
`/channel_bugyo add major #general #random #active`

`/channel_bugyo add --public major #general #random #active`

#### delete

指定したタグの channel list folder からチャンネルを削除します。

`/channel_bugyo delete [tag] [#channel_1] [#channel_2] [#channel_3] ...`

例
`/channel_bugyo delete major #general #random #active`

`/channel_bugyo delete --public major #general #random #active`


#### retrieve_bot

指定したタグの channel list folder がボットによるメッセージを収集するかを設定します。（初期値は false） \
第二引数が true であれば、ボットメッセージを収集するようになり、false であれば、ボットメッセージを無視します。

`/channel_bugyo retrieve_bot [tag] [bool]`

例
`/channel_bugyo retrieve_bot major true`

`/channel_bugyo retrieve_bot --public major true`


Channel Bugyo は自身より発せられたメッセージを無視しますが、他のボットとの兼ね合い次第では無限ループが発生しえます。

#### ch_list

指定したタグの収集対象チャンネルを羅列します。

`/channel_bugyo ch_list [tag]`

`/channel_bugyo ch_list --public [tag]`

#### tag_list

存在するタグを羅列します。

`/channel_bugyo tag_list`

### 収集先チャンネル

チャンネルにメッセージを収集するには、チャンネルに Channel Bugyo が追加されている必要があるほか、set コマンドによる設定が必要です。

#### set

Channel Bugyo が追加されているチャンネルにおいて使用することで、そのチャンネルを dist channel とし、指定したタグで収集対象となっているチャンネルのメッセージを収集します。

`/channel_bugyo set [tag_1] [tag_2] [tag_3] ...`

`/channel_bugyo set --public [tag_1] [tag_2] [tag_3] ...`

### unset

set されているタグを収集対象から外します。

`/channel_bugyo unset [tag_1] [tag_2] [tag_3] ...`

`/channel_bugyo unset --public [tag_1] [tag_2] [tag_3] ...`

#### create_channel

指定したタグを収集対象とする新たなプライベートチャンネルを作成します。

`/channel_bugyo create_channel [new_channel_name] [tag_1] [tag_2] [tag_3] ...`

`/channel_bugyo create_channel --public [new_channel_name] [tag_1] [tag_2] [tag_3] ...`

#### target_list

現在チャンネルが収集対象としているタグの一覧を表示します。

`/channel_bugyo target_list`
