## Context

現在のアプリは `Screen` enum で Main / Events / Viewer の3画面を管理する。Events 画面はログストリームを選択してから遷移するものであり、ロググループ単位での検索手段は存在しない。`aws::fetch_log_events` はすでに `start_time_ms` と `filter_pattern` を受け取るが、`end_time_ms` は未対応で、`stream_name` は必須になっている。

## Goals / Non-Goals

**Goals:**
- `g` キーでロググループ対象のログイベント検索フォームを開く
- 開始日時・終了日時・filterPattern の3フィールドを任意入力できる（デフォルト値あり）
- Enter でグループ全体を対象に `filter_log_events` API を呼び出し、結果を一覧表示する
- 一覧から `q` で検索フォームに戻れる

**Non-Goals:**
- 既存の Events 画面（ストリーム選択→Enter）の変更
- ページネーション（初回実装では対象外）
- 検索フォームの入力バリデーション（不正な日時はエラーメッセージで通知するのみ）

## Decisions

### 1. 新 Screen バリアントを2つ追加する

`Screen::EventSearch`（検索フォーム）と `Screen::GroupEvents`（検索結果）を追加する。

- **既存の `Screen::Events` を流用しない**: Events はストリーム選択が前提の状態を持つ。流用すると `q` の戻り先フラグや条件分岐が複雑になる。独立した画面として管理することで責務が明確になる。
- **`EventSearch` と `GroupEvents` を分ける**: フォーム画面と一覧画面は UI・状態・キーハンドラが別物。画面バリアントで分離する方が Rust のパターンマッチと合う。

### 2. 検索フォームのフィールド管理

`App` に以下のフィールドを追加する:

```
event_search_start: String      // デフォルト: 現在時刻 - 1時間 (例: "2025-01-01 12:00:00")
event_search_end: String        // デフォルト: 現在時刻
event_search_pattern: String    // デフォルト: "" (空文字)
event_search_focused: u8        // 0=start, 1=end, 2=pattern
```

- **Tab / Shift+Tab** でフォーカス移動（フィールド3つをサイクル）
- フォーカスしているフィールドに文字入力・Backspace が効く
- **日時フォーマット**: `YYYY-MM-DD HH:MM:SS`（ローカル時刻）で入力し、ミリ秒 Unix タイムスタンプに変換してから API へ渡す
- `q` / `Esc` で Main 画面へ戻る

### 3. aws::fetch_log_events のシグネチャ変更

- `stream_name: &str` → `stream_name: Option<&str>` に変更（None の場合は `log_stream_names` を設定しない）
- `end_time_ms: Option<i64>` パラメータを追加

既存の呼び出し箇所（ストリーム指定の `load_log_events`）は `Some(stream_name)` に変えるだけで後方互換を保てる。

### 4. GroupEvents 一覧の表示

既存の `events_screen.rs` の `draw` 関数を参考に `group_events_screen.rs` を新設する。ヘッダーにロググループ名と検索条件のサマリを表示し、ログイベント行を一覧表示する。

- `j/k` でナビゲーション
- `Enter` でイベント詳細（Viewer 画面）へ遷移（戻り先は GroupEvents）
- `q` で EventSearch 画面へ戻る

### 5. Viewer 画面の戻り先

現状 Viewer の `q` は `Screen::Events` に固定されている。GroupEvents から開いた場合も戻り先が Events になってしまう。`App` に `viewer_origin: Screen` フィールドを追加して、Viewer の `q` がそこへ戻るようにする。

## Risks / Trade-offs

- **日時パース失敗**: ユーザーが不正な日時を入力した場合、API 呼び出し前にエラーを検出してフォーム上にエラーメッセージを表示する。フォームを抜けずに修正を促す。→ `App` に `event_search_error: Option<String>` を持つ。
- **大量イベント**: `filter_log_events` はページネーション対応が必要だが、初回実装では 1 ページ目のみ取得し、後で `load_more` を追加する。
- **Viewer の戻り先変更**: 既存の `handle_viewer_key` に `viewer_origin` 参照を追加するため、既存動作を壊さないよう注意が必要。

## Open Questions

- 検索フォームの日時入力はローカル時刻とするが、タイムゾーン表示は将来的にステータスバーへ表示するか？（今回は対象外）
