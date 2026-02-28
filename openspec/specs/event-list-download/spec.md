## MODIFIED Requirements

### Requirement: ダウンロードキーで入力モード開始
EventsScreen の通常モード（非フィルター編集中）において、ユーザーが `d` キーを押すと、`download_editing` が `true` に遷移する SHALL。入力バッファのデフォルト値は EventsScreen が保持する `group_name` から生成される SHALL。

#### Scenario: dキーで入力モード開始
- **WHEN** EventsScreen でフィルター編集中でなく、`d` キーが押された
- **THEN** `download_editing` が `true` になり、デフォルトのファイル名（`{ロググループ名末尾セグメント}-{YYYY-MM-DD}.jsonl`）がバッファに設定される

### Requirement: Enterキーでファイル書き込み
EventsScreen の `handle_key` 内でダウンロードパス入力モード中に `Enter` キーが処理される際、`write_events_to_jsonl` メソッドが呼ばれる SHALL。このメソッドは EventsScreen の `impl` に定義される SHALL。

#### Scenario: Enterでファイルに書き込む
- **WHEN** EventsScreen のダウンロードパス入力モード中に `Enter` キーが押された
- **THEN** EventsScreen の `write_events_to_jsonl` メソッドが呼ばれ、`log_events.items` がJSONL形式でファイルに書き込まれる

### Requirement: 書き込み結果のステータス表示
EventsScreen の `download_status` フィールドにファイル書き込みの成功・失敗を格納する SHALL。UI描画関数は EventsScreen の `download_status` を参照してステータスバーに表示する SHALL。

#### Scenario: 書き込み成功時のメッセージ
- **WHEN** ファイルへの書き込みが成功した
- **THEN** EventsScreen の `download_status` に「Saved: {ファイルパス}」が設定される

#### Scenario: 書き込み失敗時のメッセージ
- **WHEN** ファイルへの書き込みが失敗した
- **THEN** EventsScreen の `download_status` に「Error: {エラーメッセージ}」が設定される
