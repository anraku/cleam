## Why

ロググループを選択した後、ログイベントを日時範囲と filterPattern で絞り込んで確認したい。現状はログストリームを選んでもイベントを検索・絞り込む手段がなく、調査効率が低い。

## What Changes

- ロググループ選択中に `g` キーを押すとログイベント検索フォームを開く
- 検索フォームでは開始日時・終了日時・filterPattern を入力できる（すべて任意）
- デフォルト値: 開始日時 = 現在時刻 - 1時間、終了日時 = 現在時刻、filterPattern = 空文字
- `Enter` で CloudWatch Logs の `filter_log_events` API を呼び出し、結果をログイベント一覧として表示する
- ログイベント一覧から `q` で検索フォームに戻る

## Capabilities

### New Capabilities
- `log-event-search`: ロググループを対象にした日時範囲・filterPattern によるログイベント検索と一覧表示

### Modified Capabilities

## Impact

- `src/` 以下: ログイベント検索用の新しい画面状態・UI コンポーネント・AWS API 呼び出しの追加
- AWS SDK: `filter_log_events` API の利用
- キーバインド: `g` キーをロググループ選択時のログイベント検索トリガーとして追加
