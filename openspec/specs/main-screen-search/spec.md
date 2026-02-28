## MODIFIED Requirements

### Requirement: 検索モードの起動
MainScreen の状態として `main_search_active` と `main_search_query` を保持する。MainScreen のアクティブパネルにフォーカスがある状態で `/` キーを押すと、`main_search_active` が `true` になる SHALL。

#### Scenario: グループパネルで検索モードに入る
- **WHEN** MainScreen のアクティブパネルがグループパネルの状態で `/` キーを押す
- **THEN** `main_search_active` が `true` になる

#### Scenario: ストリームパネルで検索モードに入る
- **WHEN** MainScreen のアクティブパネルがストリームパネルの状態で `/` キーを押す
- **THEN** `main_search_active` が `true` になる

### Requirement: リアルタイムフィルタリング
MainScreen の `handle_key` 内で検索モード中の文字入力を処理し、`apply_main_search` メソッドでフィルタリングを行う SHALL。これらのメソッドは MainScreen の `impl` に定義される SHALL。

#### Scenario: 文字入力によるフィルタリング
- **WHEN** MainScreen の検索モード中に文字を入力する
- **THEN** MainScreen の `apply_main_search` メソッドが呼ばれ、アクティブパネルのリストがフィルタリングされる

#### Scenario: Backspaceによる文字削除
- **WHEN** MainScreen の検索モード中に Backspace を押す
- **THEN** `main_search_query` の末尾1文字が削除され、`apply_main_search` でリストが再フィルタリングされる

### Requirement: 検索モードの終了
MainScreen の `clear_main_search` メソッドが Esc キー処理時に呼ばれ、検索状態をリセットする SHALL。

#### Scenario: Escで検索を終了する
- **WHEN** MainScreen の検索モード中に Esc を押す
- **THEN** `clear_main_search` が呼ばれ、`main_search_active` が `false` に、`main_search_query` が空に、フィルタがクリアされる
