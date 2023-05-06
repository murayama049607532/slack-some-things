# Slack Some-Things（内部向け）

## 概要

Slack Some-Thingsは、カスタマイズ可能なチャンネル収集を行うためのSlack用アプリケーションです。Channel Bugyo が追加されているチャンネル内で、以下に説明されている設定を行うことで使用可能になります。 \
現在はプレビュー版です。

コードレビューしてくれると............うれしい！！！！！！！！

## 機能

### channels list folder

channel list folder は、フォルダ毎に収集すべき対象を保存する構造体であり、メンバとして ch_list(Vec) と、ボットによるメッセージを収集するかを定める bot(bool) を持ち、ch_list_folder.json として保存されます。 \
また、フォルダはそれぞれタグと呼ばれる名前を持ち、各フォルダはこれによって区別されます。 \
アプリの権限設定については、manifest.yml を参照してください。

channel list folder に関連するコマンドとして、add、retrieve_bot、ch_list、tag_list が存在します。

#### add

指定したタグの channel list folder にチャンネルを追加します。削除機能は実装予定です。

`/channel_bugyo add [tag] [#channel_1] [#channel_2] [#channel_3] ...`

例
`/channel_bugyo add major #general #random #active`

#### retrieve_bot

指定したタグの channel list folder がボットによるメッセージを収集するかを設定します。（初期値は false） \
第二引数が true であれば、ボットメッセージを収集するようになり、それ以外の文字列が入力された場合、ボットメッセージを無視します。（修正予定）

`/channel_bugyo retrieve_bot [tag] [bool]`

例
`/channel_bugyo retrieve_bot major true`

Channel Bugyo は自身より発せられたメッセージを無視しますが、他のボットとの兼ね合い次第では無限ループが発生しえます。

#### ch_list

指定したタグの収集対象チャンネルを羅列します。

`/channel_bugyo ch_list [tag]`

#### tag_list

存在するタグを羅列します。

`/channel_bugyo tag_list`
###dist channel
dist channel は収集したメッセージを集積する目的地と収集対象のタグのリストを持つ構造体であり、ch_dists.json として保存されます。 \
dist channel にメッセージを収集するには、チャンネルに Channel Bugyo が追加されている必要があるほか、set コマンドによる設定が必要です。

#### set

Channel Bugyo が追加されているチャンネルにおいて使用することで、そのチャンネルを dist channel とし、指定したタグで収集対象となっているチャンネルのメッセージを収集します。

`/channel_bugyo set [tag_1] [tag_2] [tag_3] ...`

## todo!

#### タグ減算機能

#### プライベートタグ

