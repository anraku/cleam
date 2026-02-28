## Context

現在のイベント一覧画面（`Screen::Events`）は `handle_events_key` でキー入力を処理しており、フィルター編集時には `filter_editing: bool` + `filter_buffer: String` を使ったインライン状態管理パターンが確立されている。`App` 構造体にフラグとバッファを追加することで、同パターンを踏襲してダウンロードモードを実装できる。

`LogEvent` は `timestamp: i64` と `message: String` の2フィールドを持つ。ダウンロード対象は `log_events.items` 全体（フィルター後の表示アイテム）とする。

## Goals / Non-Goals

**Goals:**
- `d` キーでファイルパス入力モードを開始する
- デフォルトパスとして `{ロググループ名}-{YYYY-MM-DD}.jsonl`（カレントディレクトリ）を提示する
- `Enter` でファイルに JSONL 形式で書き込み、成功/失敗をステータスバーに表示する
- `Esc` でキャンセルする

**Non-Goals:**
- group_events_screen への同機能追加（別途対応）
- 書き込み先ディレクトリの自動作成
- 既存ファイルへの追記（常に上書き）
- 非同期ファイル書き込み（tokio-fs 等）

## Decisions

### 1. 状態管理: App フィールド追加（既存パターン踏襲）

`filter_editing` / `filter_buffer` と同様に、以下を `App` 構造体に追加する：

```rust
pub download_editing: bool,
pub download_path_buffer: String,
pub download_status: Option<String>,  // 成功/失敗メッセージ
```

**代替案**: 新しい `Screen::DownloadPrompt` バリアント追加
→ 却下。専用画面は不要なオーバーヘッドで、既存のインラインモードパターンで十分。

### 2. JSONL フォーマット: timestamp + message をそのまま出力

各行を以下の JSON オブジェクトとする：

```jsonl
{"timestamp":1700000000000,"message":"log line here"}
```

`serde_json` は既存 `Cargo.toml` に含まれている前提。含まれない場合は依存追加が必要。

**代替案**: 人間可読な ISO 8601 日時に変換して出力
→ 今回はシンプルに生の timestamp（ミリ秒エポック）を優先。フィルタリング・再インポートの利便性が高い。

### 3. ファイル書き込み: 同期 `std::fs`

TUI のイベントループは async だが、ファイル書き込みは短時間で完了するため `std::fs::write` で問題ない。

**代替案**: `tokio::fs::write` で非同期化
→ イベント数が通常数百〜数千件程度であれば同期で十分。ブロック時間は無視できる。

### 4. デフォルトファイル名生成

ロググループ名は `last_selected_group` のインデックスから `log_groups.items` を参照して取得する。日付は `chrono` の `Local::now()` で生成する（`chrono` は既存依存）。

ロググループ名にスラッシュ（`/`）が含まれる場合はハイフン（`-`）に置換してファイル名を安全にする。

### 5. ステータス表示

`download_status: Option<String>` に書き込み結果メッセージを保存し、`events_screen` の描画関数でステータスバーに表示する。次のキー操作時にクリアする。

## Risks / Trade-offs

- **大量イベント時のメモリ**: `visible_items()` で取得した全イベントを一度にシリアライズするため、件数が多いと一時的にメモリ使用量が増加する → 通常の CloudWatch Logs ページネーション範囲（数千件）では許容範囲内
- **ロググループ名取得**: `last_selected_group` が `None` の場合（直接 URL 遷移等）はフォールバックとして `"unknown"` を使用する
- **serde_json 依存**: 未追加の場合は `Cargo.toml` への追記が必要
