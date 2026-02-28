## MODIFIED Requirements

### Requirement: 検索フォームの起動
メイン画面でロググループにカーソルが当たっている状態で `g` キーを押すと、ログイベント検索フォーム画面へ遷移する。
ロググループが1件も存在しない、またはカーソルが当たっていない場合は何もしない。
MainScreenの `handle_key` は `ScreenAction::Navigate(NavigateTo::EventSearch { group_name })` を返す SHALL。

#### Scenario: グループ選択中にgキーで検索フォームへ遷移する
- **WHEN** メイン画面でロググループにカーソルが当たっている状態で `g` を押す
- **THEN** MainScreenの `handle_key` が `NavigateTo::EventSearch` を返し、選択中のグループ名が含まれる

#### Scenario: グループ未選択時はgキーを無視する
- **WHEN** メイン画面でロググループが空またはカーソルが当たっていない状態で `g` を押す
- **THEN** `ScreenAction::None` が返され、画面遷移は起こらない

### Requirement: 検索フォームのデフォルト値
EventSearchScreen が作成される際、各フィールドに以下のデフォルト値が設定される。
- 開始日時: 現在時刻の1時間前（`YYYY-MM-DD HH:MM:SS` 形式、ローカル時刻）
- 終了日時: 現在時刻（`YYYY-MM-DD HH:MM:SS` 形式、ローカル時刻）
- filterPattern: 空文字
EventSearchScreen は `group_name` フィールドに対象のロググループ名を保持する SHALL。

#### Scenario: EventSearchScreen作成時に開始日時にデフォルト値が入る
- **WHEN** EventSearchScreen が `group_name` を引数に作成される
- **THEN** 開始日時フィールドに現在時刻 - 1時間が `YYYY-MM-DD HH:MM:SS` 形式で設定される

#### Scenario: EventSearchScreen作成時に終了日時にデフォルト値が入る
- **WHEN** EventSearchScreen が作成される
- **THEN** 終了日時フィールドに現在時刻が `YYYY-MM-DD HH:MM:SS` 形式で設定される

#### Scenario: EventSearchScreen作成時にfilterPatternは空
- **WHEN** EventSearchScreen が作成される
- **THEN** filterPattern フィールドは空文字である

### Requirement: 検索の実行
EventSearchScreen の `handle_key` で `Enter` キーが処理される際、日時バリデーション後に `ScreenAction::Navigate(NavigateTo::GroupEvents { group_name, start_ms, end_ms, pattern })` を返す SHALL。
日時として解釈できない文字列が入力されている場合はエラーメッセージを `event_search_error` に設定し、`ScreenAction::None` を返す SHALL。

#### Scenario: 有効な条件でEnterを押すとNavigateアクションが返る
- **WHEN** EventSearchScreen で `Enter` を押す（日時が有効または空）
- **THEN** `ScreenAction::Navigate(NavigateTo::GroupEvents { ... })` が返される

#### Scenario: 不正な日時でEnterを押すとエラーを表示する
- **WHEN** 日時フィールドに `YYYY-MM-DD HH:MM:SS` として解釈できない文字列が入力された状態で `Enter` を押す
- **THEN** `event_search_error` にエラーメッセージが設定され、`ScreenAction::None` が返される

### Requirement: 検索結果一覧から検索フォームへの戻り
GroupEventsScreen の `handle_key` で `q` キーが処理される際、`ScreenAction::Navigate(NavigateTo::EventSearch { group_name })` を返す SHALL。

#### Scenario: qキーでEventSearch画面に戻る
- **WHEN** GroupEventsScreen で `q` を押す
- **THEN** `ScreenAction::Navigate(NavigateTo::EventSearch { group_name })` が返される
