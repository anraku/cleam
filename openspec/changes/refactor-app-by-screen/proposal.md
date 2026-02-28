## Why

App構造体が25個のフィールドと21個のメソッドを持ち、全スクリーンの状態とロジックが1つの構造体に集約されている。スクリーンが増えるたびにAppが肥大化し、変更の影響範囲が把握しにくくなっている。UIの描画は既に`src/ui/`にスクリーン別で分離されているが、状態管理とキーハンドリングが`App`に集中しているため、関心の分離が不十分。

## What Changes

- **BREAKING**: `App`構造体を分解し、各スクリーン固有の状態をスクリーン別の構造体に分離する
  - `MainScreen`: `log_groups`, `main_search_query`, `main_search_active`, `filter_input`, `filter_editing`, `filter_buffer`
  - `EventsScreen`: `log_streams`, `log_events`, `selected_event`, `last_selected_group`, `active_panel`
  - `ViewerScreen`: `viewer_scroll`, `viewer_origin`
  - `EventSearchScreen`: `event_search_start`, `event_search_end`, `event_search_pattern`, `event_search_focused`, `event_search_error`
  - `GroupEventsScreen`: `log_events`（グループイベント用）, `download_editing`, `download_path_buffer`, `download_status`
- AWS SDK の `Client` を `Arc<Client>` で包み、各スクリーン構造体から共有参照できるようにする
- 各スクリーン構造体に対応する `handle_key` メソッドとデータロードメソッドを移動する
- `App` はスクリーン遷移の管理、共有リソース（`Arc<Client>`）の保持、スクリーン間のディスパッチに責務を限定する

## Capabilities

### New Capabilities

- `screen-state-separation`: 各スクリーンの状態を独立した構造体として定義し、スクリーン固有のロジック（キーハンドリング、データロード）をそのスクリーンの`impl`に移動する仕組み
- `shared-client`: AWS SDK Clientを`Arc`で包み、複数のスクリーン構造体から共有参照可能にするパターン

### Modified Capabilities

- `log-event-search`: EventSearchScreenの状態とキーハンドリングが独立した構造体に移動する
- `main-screen-search`: MainScreenの検索状態とロジックが独立した構造体に移動する
- `event-list-download`: ダウンロード関連の状態とロジックがGroupEventsScreenに移動する

## Impact

- **コード**: `src/app.rs`の大幅な分割。各スクリーンのモジュール（例: `src/screen/main.rs`）を新規作成
- **既存UI**: `src/ui/`の各描画関数のシグネチャが変更される（`&App`から各スクリーン構造体への参照に変更）
- **依存関係**: `aws-sdk-cloudwatchlogs`の`Client`を`Arc`で包むため、`Arc`のインポートが追加される
- **テスト**: 各スクリーンを独立してテスト可能になる（テスタビリティの向上）
