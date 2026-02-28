## ADDED Requirements

### Requirement: スクリーン別の状態構造体
各スクリーン（Main, Events, Viewer, EventSearch, GroupEvents）は独立した構造体として定義される SHALL。各構造体はそのスクリーンに固有のフィールドのみを保持する SHALL。

#### Scenario: MainScreen構造体がメイン画面固有の状態を保持する
- **WHEN** MainScreen構造体が定義される
- **THEN** `log_groups`, `log_streams`, `active_panel`, `main_search_query`, `main_search_active`, `filter_input`, `filter_editing`, `filter_buffer`, `last_selected_group` のフィールドを保持する

#### Scenario: EventsScreen構造体がイベント一覧固有の状態を保持する
- **WHEN** EventsScreen構造体が定義される
- **THEN** `log_events`, `selected_event`, `filter_input`, `filter_editing`, `filter_buffer`, `download_editing`, `download_path_buffer`, `download_status` のフィールドを保持する

#### Scenario: ViewerScreen構造体がビューア固有の状態を保持する
- **WHEN** ViewerScreen構造体が定義される
- **THEN** `viewer_scroll`, `selected_event`, `origin`（戻り先スクリーン識別子）のフィールドを保持する

#### Scenario: EventSearchScreen構造体が検索フォーム固有の状態を保持する
- **WHEN** EventSearchScreen構造体が定義される
- **THEN** `event_search_start`, `event_search_end`, `event_search_pattern`, `event_search_focused`, `event_search_error`, `group_name` のフィールドを保持する

#### Scenario: GroupEventsScreen構造体がグループイベント一覧固有の状態を保持する
- **WHEN** GroupEventsScreen構造体が定義される
- **THEN** `log_events`, `selected_event`, `group_name` のフィールドを保持する

### Requirement: スクリーン別のキーハンドリング
各スクリーン構造体は自身の `handle_key` メソッドを持つ SHALL。`handle_key` は `ScreenAction` enum を返す SHALL。

#### Scenario: 各スクリーンのhandle_keyがScreenActionを返す
- **WHEN** 任意のスクリーンの `handle_key` メソッドが呼ばれる
- **THEN** `ScreenAction` enum（None, Quit, Navigate, NeedsClear のいずれか）を返す

#### Scenario: スクリーン遷移はNavigateアクションで通知する
- **WHEN** スクリーン内の操作で別スクリーンへの遷移が必要になる
- **THEN** `ScreenAction::Navigate(NavigateTo::*)` を返し、遷移先と必要なデータを含める

### Requirement: CurrentScreen enumによるアクティブスクリーン管理
`App` は `CurrentScreen` enum でアクティブなスクリーンの構造体を保持する SHALL。同時に複数のスクリーンがアクティブになることはない SHALL。

#### Scenario: Appが1つのアクティブスクリーンのみを保持する
- **WHEN** `App` の `CurrentScreen` フィールドが参照される
- **THEN** 正確に1つのスクリーン構造体のバリアントが保持されている

#### Scenario: スクリーン遷移時に新しいスクリーン構造体が作成される
- **WHEN** `ScreenAction::Navigate` が処理される
- **THEN** `CurrentScreen` が新しいスクリーン構造体のバリアントに置き換わる

### Requirement: スクリーン別のデータロードメソッド
AWS APIを呼び出すデータロードメソッドは、該当するスクリーンの構造体の `impl` に定義される SHALL。

#### Scenario: MainScreenがロググループとストリームのロードメソッドを持つ
- **WHEN** MainScreen が作成される
- **THEN** `load_log_groups`, `load_more_groups`, `load_log_streams`, `load_more_streams` メソッドが利用可能

#### Scenario: EventsScreenがログイベントのロードメソッドを持つ
- **WHEN** EventsScreen が作成される
- **THEN** `load_log_events`, `load_more_events` メソッドが利用可能

#### Scenario: GroupEventsScreenがグループイベントのロードメソッドを持つ
- **WHEN** GroupEventsScreen が作成される
- **THEN** `load_group_events` メソッドが利用可能

### Requirement: スクリーンモジュールのファイル配置
各スクリーン構造体は `src/screen/` ディレクトリ配下に配置される SHALL。`src/screen/mod.rs` に共通の型定義（`ScreenAction`, `NavigateTo`, `ScreenId`）を置く SHALL。

#### Scenario: screenモジュールが正しいファイル構成を持つ
- **WHEN** `src/screen/` ディレクトリが参照される
- **THEN** `mod.rs`, `main.rs`, `events.rs`, `viewer.rs`, `event_search.rs`, `group_events.rs` が存在する

### Requirement: UI描画関数のシグネチャ変更
`src/ui/` の各描画関数は `&App` ではなく、対応するスクリーン構造体の参照を引数に取る SHALL。

#### Scenario: main_screen::drawがMainScreenを受け取る
- **WHEN** `ui::main_screen::draw` が呼び出される
- **THEN** 引数は `&mut Frame` と `&MainScreen` である

#### Scenario: ui::drawがCurrentScreenでディスパッチする
- **WHEN** `ui::draw` が呼び出される
- **THEN** `CurrentScreen` の各バリアントに対して対応する描画関数を呼び出す
