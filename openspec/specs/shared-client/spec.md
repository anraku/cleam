## ADDED Requirements

### Requirement: Arc<Client>による共有
AWS SDK の `Client` は `Arc<Client>` として保持される SHALL。`App` と各スクリーン構造体が同じ `Client` インスタンスを共有参照する SHALL。

#### Scenario: Appの初期化時にArc<Client>が作成される
- **WHEN** `App::new()` が `Client` を受け取る
- **THEN** `Arc::new(client)` で包んで保持する

#### Scenario: スクリーン構造体の生成時にArc<Client>がクローンされる
- **WHEN** 新しいスクリーン構造体が作成される
- **THEN** `App` が保持する `Arc<Client>` の `clone()` がスクリーン構造体に渡される

#### Scenario: 各スクリーンからAWS APIを呼び出せる
- **WHEN** スクリーン構造体のデータロードメソッドが呼ばれる
- **THEN** 自身が保持する `Arc<Client>` を使って `aws::fetch_*` 関数を呼び出す
