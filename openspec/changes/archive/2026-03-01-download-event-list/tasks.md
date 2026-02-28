## 1. App 状態の追加

- [x] 1.1 `App` 構造体に `download_editing: bool`、`download_path_buffer: String`、`download_status: Option<String>` フィールドを追加する
- [x] 1.2 `App::new()` でこれらフィールドを初期化する（`download_editing: false`、`download_path_buffer: String::new()`、`download_status: None`）

## 2. デフォルトファイル名生成ロジック

- [x] 2.1 現在のロググループ名を `last_selected_group` + `log_groups.items` から取得するヘルパーを実装する（取得できない場合は `"unknown"` を返す）
- [x] 2.2 ロググループ名の `/` を `-` に置換する処理を追加する
- [x] 2.3 `jiff` を使って現在日付を `YYYY-MM-DD` 形式で取得し、`{ロググループ名}-{現在日時}.jsonl` の文字列を生成するロジックを実装する

## 3. キーハンドリングの更新

- [x] 3.1 `handle_events_key` の通常モード（`else` ブロック）に `KeyCode::Char('d')` の処理を追加する：`download_editing = true` にし、デフォルトファイル名をバッファにセットする
- [x] 3.2 `handle_events_key` の先頭に `download_editing` が `true` の場合のブロックを追加する：
  - `Enter`: ファイル書き込みを実行し `download_editing = false` にする
  - `Esc`: `download_editing = false`、`download_path_buffer` をクリアする
  - `Backspace`: バッファの末尾を削除する
  - `Char(c)`: バッファに文字を追加する

## 4. JSONL ファイル書き込み処理

- [x] 4.1 `log_events.items` を受け取り、各 `LogEvent` を `{"timestamp": ..., "message": "..."}` の JSON 行に変換して連結する関数を実装する
- [x] 4.2 `std::fs::write` でファイルに書き込む処理を実装し、成功時は `download_status = Some(format!("Saved: {}", path))`、失敗時は `download_status = Some(format!("Error: {}", e))` をセットする

## 5. UI への表示

- [x] 5.1 `events_screen.rs` の `draw` 関数でダウンロードパス入力モード時にファイルパスバッファをフッター/ステータスバーに表示する（フィルター編集表示と同様のパターンで実装する）
- [x] 5.2 `download_status` が `Some` の場合にステータスメッセージを表示する
- [x] 5.3 次のキー操作が来たタイミングで `download_status` をクリアする（`handle_events_key` の先頭で `download_status = None` にする）
