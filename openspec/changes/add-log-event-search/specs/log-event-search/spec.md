## ADDED Requirements

### Requirement: 検索フォームの起動
メイン画面でロググループにカーソルが当たっている状態で `g` キーを押すと、ログイベント検索フォーム画面へ遷移する。
ロググループが1件も存在しない、またはカーソルが当たっていない場合は何もしない。

#### Scenario: グループ選択中にgキーで検索フォームへ遷移する
- **WHEN** メイン画面でロググループにカーソルが当たっている状態で `g` を押す
- **THEN** ログイベント検索フォーム画面（EventSearch）に遷移する

#### Scenario: グループ未選択時はgキーを無視する
- **WHEN** メイン画面でロググループが空またはカーソルが当たっていない状態で `g` を押す
- **THEN** 画面遷移は起こらない

### Requirement: 検索フォームのデフォルト値
検索フォームを開いた時点で、各フィールドに以下のデフォルト値が設定される。
- 開始日時: 現在時刻の1時間前（`YYYY-MM-DD HH:MM:SS` 形式、ローカル時刻）
- 終了日時: 現在時刻（`YYYY-MM-DD HH:MM:SS` 形式、ローカル時刻）
- filterPattern: 空文字

#### Scenario: 検索フォームを開いたとき開始日時にデフォルト値が入る
- **WHEN** EventSearch 画面に遷移する
- **THEN** 開始日時フィールドに現在時刻 - 1時間が `YYYY-MM-DD HH:MM:SS` 形式で表示される

#### Scenario: 検索フォームを開いたとき終了日時にデフォルト値が入る
- **WHEN** EventSearch 画面に遷移する
- **THEN** 終了日時フィールドに現在時刻が `YYYY-MM-DD HH:MM:SS` 形式で表示される

#### Scenario: 検索フォームを開いたとき filterPattern は空
- **WHEN** EventSearch 画面に遷移する
- **THEN** filterPattern フィールドは空文字である

### Requirement: 検索フォームのフィールド入力
検索フォームには「開始日時」「終了日時」「filterPattern」の3フィールドがある。
`Tab` キーで次のフィールドへ、`Shift+Tab` で前のフィールドへフォーカスを移動できる。
フォーカス中のフィールドに文字入力・`Backspace` による削除ができる。

#### Scenario: Tabキーでフォーカスが次フィールドへ移動する
- **WHEN** 任意のフィールドにフォーカスがある状態で `Tab` を押す
- **THEN** フォーカスが次のフィールドへ移動する（filterPattern の次は開始日時に戻る）

#### Scenario: Shift+Tabキーでフォーカスが前フィールドへ移動する
- **WHEN** 任意のフィールドにフォーカスがある状態で `Shift+Tab` を押す
- **THEN** フォーカスが前のフィールドへ移動する（開始日時の前は filterPattern に戻る）

#### Scenario: フォーカス中フィールドに文字が入力される
- **WHEN** フォーカス中のフィールドで文字キーを押す
- **THEN** そのフィールドの末尾に文字が追加される

#### Scenario: フォーカス中フィールドの末尾文字を削除できる
- **WHEN** フォーカス中のフィールドで `Backspace` を押す
- **THEN** そのフィールドの末尾1文字が削除される

### Requirement: 検索の実行
`Enter` キーを押すと入力された条件で CloudWatch Logs の `filter_log_events` API を呼び出す。
対象はフォームを開いた時点で選択されていたロググループ全体（ストリーム指定なし）。
空の開始日時・終了日時は API に渡さない（無制限）。
日時として解釈できない文字列が入力されている場合はエラーメッセージをフォーム上に表示し、画面遷移しない。

#### Scenario: 有効な条件でEnterを押すとAPI呼び出しが行われる
- **WHEN** 検索フォームで `Enter` を押す（日時が有効または空）
- **THEN** 選択ロググループを対象に `filter_log_events` API が呼び出され、GroupEvents 画面へ遷移する

#### Scenario: 開始日時が空の場合はstart_timeを渡さない
- **WHEN** 開始日時フィールドが空の状態で `Enter` を押す
- **THEN** API に `start_time` パラメータを渡さずに呼び出す

#### Scenario: 終了日時が空の場合はend_timeを渡さない
- **WHEN** 終了日時フィールドが空の状態で `Enter` を押す
- **THEN** API に `end_time` パラメータを渡さずに呼び出す

#### Scenario: filterPatternが空の場合はパターンを渡さない
- **WHEN** filterPattern フィールドが空の状態で `Enter` を押す
- **THEN** API に `filter_pattern` パラメータを渡さずに呼び出す

#### Scenario: 不正な日時でEnterを押すとエラーを表示する
- **WHEN** 日時フィールドに `YYYY-MM-DD HH:MM:SS` として解釈できない文字列が入力された状態で `Enter` を押す
- **THEN** フォーム上にエラーメッセージが表示され、GroupEvents 画面へは遷移しない

### Requirement: 検索フォームからメイン画面への戻り
`q` または `Esc` を押すとメイン画面に戻る。

#### Scenario: qキーでメイン画面に戻る
- **WHEN** EventSearch 画面で `q` を押す
- **THEN** メイン画面（Main）へ遷移する

#### Scenario: Escキーでメイン画面に戻る
- **WHEN** EventSearch 画面で `Esc` を押す
- **THEN** メイン画面（Main）へ遷移する

### Requirement: ログイベント検索結果の一覧表示
検索が成功すると GroupEvents 画面へ遷移し、ログイベントを一覧で表示する。
画面ヘッダーにはロググループ名と検索条件のサマリを表示する。
イベントがない場合は「No events found」と表示する。

#### Scenario: 検索結果がある場合にイベント一覧を表示する
- **WHEN** `filter_log_events` API がイベントを返す
- **THEN** GroupEvents 画面にタイムスタンプとメッセージの一覧が表示される

#### Scenario: 検索結果がない場合にメッセージを表示する
- **WHEN** `filter_log_events` API がイベント0件を返す
- **THEN** GroupEvents 画面に「No events found」と表示される

#### Scenario: ヘッダーに検索条件のサマリが表示される
- **WHEN** GroupEvents 画面が表示される
- **THEN** ヘッダーにロググループ名と指定した開始・終了日時・filterPattern が表示される

### Requirement: 検索結果一覧のナビゲーション
GroupEvents 画面では `j` / `k` キーでイベント間をナビゲーションできる。

#### Scenario: jキーで次のイベントへ移動する
- **WHEN** GroupEvents 画面で `j` または `↓` を押す
- **THEN** カーソルが次のイベントへ移動する

#### Scenario: kキーで前のイベントへ移動する
- **WHEN** GroupEvents 画面で `k` または `↑` を押す
- **THEN** カーソルが前のイベントへ移動する

### Requirement: 検索結果からイベント詳細への遷移
GroupEvents 画面で `Enter` を押すと選択中のイベントを Viewer 画面で表示する。
Viewer から `q` を押すと GroupEvents 画面へ戻る。

#### Scenario: EnterキーでViewer画面へ遷移する
- **WHEN** GroupEvents 画面でイベントにカーソルが当たっている状態で `Enter` を押す
- **THEN** Viewer 画面でそのイベントのメッセージ全文が表示される

#### Scenario: ViewerからqキーでGroupEvents画面へ戻る
- **WHEN** GroupEvents から開いた Viewer 画面で `q` を押す
- **THEN** GroupEvents 画面へ戻る

### Requirement: 検索結果一覧から検索フォームへの戻り
GroupEvents 画面で `q` を押すと EventSearch 画面へ戻る。
検索フォームには直前の入力値が保持されている。

#### Scenario: qキーでEventSearch画面に戻る
- **WHEN** GroupEvents 画面で `q` を押す
- **THEN** EventSearch 画面へ遷移し、直前の検索条件が入力済みの状態で表示される
