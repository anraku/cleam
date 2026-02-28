## 1. AWS層の変更

- [x] 1.1 `aws::fetch_log_events` の `stream_name` 引数を `Option<&str>` に変更し、`None` のとき `log_stream_names` を設定しないようにする
- [x] 1.2 `aws::fetch_log_events` に `end_time_ms: Option<i64>` 引数を追加し、Some のとき `end_time` を設定する
- [x] 1.3 既存の `app.rs` 内の `fetch_log_events` 呼び出し箇所（`load_log_events`）を `stream_name: Some(...)` に更新してコンパイルを通す

## 2. App の画面状態・フィールドの追加

- [x] 2.1 `Screen` enum に `EventSearch` と `GroupEvents` を追加する
- [x] 2.2 `App` に検索フォームフィールドを追加する: `event_search_start: String`, `event_search_end: String`, `event_search_pattern: String`, `event_search_focused: u8`, `event_search_error: Option<String>`
- [x] 2.3 `App` に `viewer_origin: Screen` フィールドを追加し、デフォルト値を `Screen::Events` にする
- [x] 2.4 `App::new` で各フィールドの初期値を設定する（`event_search_*` はデフォルト値を設定、`viewer_origin` は `Screen::Events`）

## 3. App のキー処理の追加・変更

- [x] 3.1 `handle_main_key` に `g` キーのハンドラを追加する: ロググループが選択中の場合に `event_search_start/end` を現在時刻ベースで初期化し `Screen::EventSearch` へ遷移する
- [x] 3.2 `handle_event_search_key` メソッドを新規追加する: Tab/Shift+Tab でフォーカス移動、文字入力・Backspace をフォーカス中フィールドに適用、`q`/`Esc` で `Screen::Main` へ戻る
- [x] 3.3 `handle_event_search_key` の `Enter` 処理を実装する: 日時文字列をパースし失敗時は `event_search_error` にメッセージを設定して返す、成功時は `load_group_events` を呼び出し `Screen::GroupEvents` へ遷移する
- [x] 3.4 `handle_group_events_key` メソッドを新規追加する: `j/k`・`↑/↓` でナビゲーション、`Enter` で `viewer_origin = Screen::GroupEvents` を設定し Viewer へ遷移、`q` で `Screen::EventSearch` へ戻る
- [x] 3.5 `handle_viewer_key` の `q` 処理を `self.screen = self.viewer_origin.clone()` に変更する
- [x] 3.6 `handle_key` のマッチアームに `Screen::EventSearch` と `Screen::GroupEvents` を追加する

## 4. App のログイベント取得処理の追加

- [x] 4.1 `load_group_events` メソッドを新規追加する: `event_search_start/end` をミリ秒タイムスタンプに変換し、`stream_name: None` で `aws::fetch_log_events` を呼び出し `log_events` に格納する

## 5. UI - EventSearch 画面

- [x] 5.1 `src/ui/event_search_screen.rs` を新規作成し、`draw` 関数を実装する
- [x] 5.2 フォームレイアウトを実装する: 3フィールド（開始日時・終了日時・filterPattern）を縦並びで表示し、フォーカス中フィールドをハイライト表示する
- [x] 5.3 エラーメッセージエリアを実装する: `event_search_error` が `Some` のとき赤字でフォーム下部に表示する
- [x] 5.4 操作ヒント（Tab: 移動, Enter: 検索, q: 戻る）をフッターに表示する

## 6. UI - GroupEvents 画面

- [x] 6.1 `src/ui/group_events_screen.rs` を新規作成し、`draw` 関数を実装する
- [x] 6.2 ヘッダーにロググループ名と検索条件のサマリ（開始・終了日時、filterPattern）を表示する
- [x] 6.3 ログイベント一覧を `events_screen.rs` と同様のスタイルでリスト表示する
- [x] 6.4 イベントが0件の場合に「No events found」を表示する
- [x] 6.5 操作ヒント（j/k: 移動, Enter: 詳細, q: 検索に戻る）をフッターに表示する

## 7. UI モジュールの統合

- [x] 7.1 `src/ui/mod.rs` に `event_search_screen` と `group_events_screen` モジュールを追加する
- [x] 7.2 `ui::draw` の `match app.screen` に `Screen::EventSearch` と `Screen::GroupEvents` のアームを追加する
