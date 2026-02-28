## 1. 共通型の定義とモジュール構造の作成

- [ ] 1.1 `src/screen/mod.rs` を作成し、`ScreenAction`, `NavigateTo`, `ScreenId` enum を定義する
- [ ] 1.2 `src/screen/` 配下に `main.rs`, `events.rs`, `viewer.rs`, `event_search.rs`, `group_events.rs` の空ファイルを作成し、`mod.rs` でサブモジュールとして宣言する
- [ ] 1.3 `src/main.rs` に `mod screen;` を追加する

## 2. スクリーン構造体の定義

- [ ] 2.1 `src/screen/main.rs` に `MainScreen` 構造体を定義する（`client: Arc<Client>`, `log_groups`, `log_streams`, `active_panel`, `main_search_query`, `main_search_active`, `filter_input`, `filter_editing`, `filter_buffer`, `last_selected_group`, `needs_clear`）
- [ ] 2.2 `src/screen/events.rs` に `EventsScreen` 構造体を定義する（`client: Arc<Client>`, `log_events`, `selected_event`, `filter_input`, `filter_editing`, `filter_buffer`, `download_editing`, `download_path_buffer`, `download_status`, `group_name`, `stream_name`, `needs_clear`）
- [ ] 2.3 `src/screen/viewer.rs` に `ViewerScreen` 構造体を定義する（`viewer_scroll`, `selected_event`, `origin: ScreenId`）
- [ ] 2.4 `src/screen/event_search.rs` に `EventSearchScreen` 構造体を定義する（`client: Arc<Client>`, `event_search_start`, `event_search_end`, `event_search_pattern`, `event_search_focused`, `event_search_error`, `group_name`）
- [ ] 2.5 `src/screen/group_events.rs` に `GroupEventsScreen` 構造体を定義する（`client: Arc<Client>`, `log_events`, `selected_event`, `group_name`, `needs_clear`）

## 3. キーハンドリングメソッドの移動

- [ ] 3.1 `App::handle_main_key` のロジックを `MainScreen::handle_key` に移動する（`ScreenAction` を返すよう変更）
- [ ] 3.2 `App::apply_main_search` と `App::clear_main_search` を `MainScreen` の `impl` に移動する
- [ ] 3.3 `App::handle_events_key` のロジックを `EventsScreen::handle_key` に移動する
- [ ] 3.4 `App::handle_viewer_key` のロジックを `ViewerScreen::handle_key` に移動する
- [ ] 3.5 `App::handle_event_search_key` のロジックを `EventSearchScreen::handle_key` に移動する
- [ ] 3.6 `App::handle_group_events_key` のロジックを `GroupEventsScreen::handle_key` に移動する

## 4. データロードメソッドの移動

- [ ] 4.1 `App::load_log_groups`, `App::load_more_groups`, `App::load_log_streams`, `App::load_more_streams` を `MainScreen` の `impl` に移動する（`&self.client` → `&self.client` as `Arc<Client>`）
- [ ] 4.2 `App::load_log_events`, `App::load_more_events` を `EventsScreen` の `impl` に移動する
- [ ] 4.3 `App::load_group_events` を `GroupEventsScreen` の `impl` に移動する
- [ ] 4.4 `App::write_events_to_jsonl`, `App::default_download_path`, `App::current_log_group_name` を `EventsScreen` の `impl` に移動する

## 5. App構造体のリファクタリング

- [ ] 5.1 `App` 構造体を `client: Arc<Client>`, `screen: CurrentScreen`, `needs_clear: bool` のみに変更する
- [ ] 5.2 `App::new()` を `Arc::new(client)` を使うよう変更し、初期状態として `CurrentScreen::Main(MainScreen::new(...))` を返すようにする
- [ ] 5.3 `App::run()` のイベントループを `CurrentScreen` に対するマッチングでディスパッチするよう変更する
- [ ] 5.4 `App::run()` に `ScreenAction` ハンドリングを実装する（Navigate時のスクリーン構造体生成とCurrentScreen切り替え）
- [ ] 5.5 `App::check_group_change` を `MainScreen::check_group_change` に移動する
- [ ] 5.6 `App::check_pagination` のロジックを各スクリーンの `check_pagination` メソッドに分散する

## 6. UI描画関数のシグネチャ変更

- [ ] 6.1 `src/ui/main_screen.rs` の `draw` シグネチャを `(f: &mut Frame, screen: &MainScreen)` に変更し、`app.` 参照を `screen.` に置き換える
- [ ] 6.2 `src/ui/events_screen.rs` の `draw` シグネチャを `(f: &mut Frame, screen: &EventsScreen)` に変更する
- [ ] 6.3 `src/ui/viewer_screen.rs` の `draw` シグネチャを `(f: &mut Frame, screen: &ViewerScreen)` に変更する
- [ ] 6.4 `src/ui/event_search_screen.rs` の `draw` シグネチャを `(f: &mut Frame, screen: &EventSearchScreen)` に変更する
- [ ] 6.5 `src/ui/group_events_screen.rs` の `draw` シグネチャを `(f: &mut Frame, screen: &GroupEventsScreen)` に変更する
- [ ] 6.6 `src/ui/mod.rs` の `draw` 関数を `CurrentScreen` に対するマッチングでディスパッチするよう変更する

## 7. 不要コードの削除とコンパイル確認

- [ ] 7.1 `src/app.rs` から移動済みのメソッドとフィールドを削除する（`Screen` enum, `ActivePanel` enum は `src/screen/mod.rs` に移動）
- [ ] 7.2 `StatefulList<T>`, `LogGroup`, `LogStream`, `LogEvent` を `src/app.rs` に残すか共通モジュールに移動するか整理する
- [ ] 7.3 `cargo build` でコンパイルが通ることを確認する
- [ ] 7.4 `cargo clippy` で警告がないことを確認する
- [ ] 7.5 アプリケーションを手動で起動し、全スクリーンの遷移・操作が正常に動作することを確認する
