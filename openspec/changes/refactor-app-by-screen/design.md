## Context

現在のアプリケーションは `App` 構造体（25フィールド、21メソッド、約570行の`impl`ブロック）に全スクリーンの状態とロジックが集約されている。UIの描画は `src/ui/` にスクリーン別ファイルとして分離済みだが、すべての描画関数は `&mut App` を引数に取っており、各スクリーンが必要としないフィールドにもアクセス可能な状態。

スクリーン構成:
- **Main**: ロググループ・ストリーム一覧、パネル切替、検索
- **Events**: ログイベント一覧、フィルタ、ダウンロード
- **Viewer**: 選択したイベントの詳細表示（スクロール）
- **EventSearch**: グループ内イベントの日時・パターン検索フォーム
- **GroupEvents**: グループ検索結果のイベント一覧

AWS SDK `Client` は `App::new()` で受け取り、各 `load_*` メソッドで `&self.client` として使用している。

## Goals / Non-Goals

**Goals:**
- 各スクリーンの状態を独立した構造体に分離し、関心の分離を実現する
- `Arc<Client>` で AWS SDK クライアントを共有し、各スクリーンから利用可能にする
- 各スクリーンに `handle_key` とデータロードメソッドを移動する
- `App` をスクリーン遷移のコーディネーターに限定する
- UIの描画関数シグネチャを各スクリーン構造体参照に変更する

**Non-Goals:**
- 非同期タスク（tokio spawn）によるバックグラウンドロードの導入（将来の改善）
- UI描画ロジックの変更（表示内容・レイアウトは変更しない）
- `StatefulList<T>`, `LogGroup`, `LogStream`, `LogEvent` の構造変更
- aws.rs のAPI関数のシグネチャ変更

## Decisions

### 1. スクリーン構造体の導入とファイル配置

**決定**: `src/screen/` ディレクトリを新設し、各スクリーンを独立モジュールとして配置する。

```
src/screen/
  mod.rs          # Screen enum, ScreenAction, 共通トレイト
  main.rs         # MainScreen
  events.rs       # EventsScreen
  viewer.rs       # ViewerScreen
  event_search.rs # EventSearchScreen
  group_events.rs # GroupEventsScreen
```

**理由**: `src/ui/` は描画専用として維持し、状態+ロジックは `src/screen/` に分離することで責務が明確になる。

**代替案**: `src/ui/` にロジックも統合する案 → 描画とロジックが混在し、現状の `app.rs` の問題が移動するだけになるため不採用。

### 2. スクリーン間のデータ受け渡し — アクション型による遷移

**決定**: 各スクリーンの `handle_key` は `ScreenAction` enum を返し、`App` がそれに基づいてスクリーン遷移やデータの受け渡しを行う。

```rust
pub enum ScreenAction {
    None,
    Quit,
    Navigate(NavigateTo),
    NeedsClear,
}

pub enum NavigateTo {
    Main,
    Events { group_name: String, stream_name: String, filter: Option<String> },
    Viewer { event: LogEvent, origin: ScreenId },
    EventSearch { group_name: String },
    GroupEvents { group_name: String, start_ms: Option<i64>, end_ms: Option<i64>, pattern: Option<String> },
}
```

**理由**: スクリーン間の依存を `App` に集約し、各スクリーンは自身の遷移先に必要なデータを `NavigateTo` に含めるだけで済む。スクリーンが互いを直接知る必要がない。

**代替案**: スクリーン同士が直接参照し合う案 → 循環依存のリスクがあるため不採用。

### 3. `Arc<Client>` の共有パターン

**決定**: `Arc<Client>` を `App` が保持し、各スクリーン構造体の生成時にクローンして渡す。

```rust
pub struct App {
    client: Arc<Client>,
    screen: CurrentScreen,
    needs_clear: bool,
}

pub struct MainScreen {
    client: Arc<Client>,
    log_groups: StatefulList<LogGroup>,
    log_streams: StatefulList<LogStream>,
    // ...
}
```

**理由**: `Arc<Client>` は `Clone` が安価（参照カウントのインクリメントのみ）であり、各スクリーンが独立して `client` を保持することでライフタイムの問題を回避できる。

**代替案**: `&Client` 参照を渡す案 → ライフタイムが複雑になり、スクリーン遷移時に `App` のミュータブル借用と競合するため不採用。

### 4. CurrentScreen の表現 — enum でスクリーン構造体を保持

**決定**: `App` は `CurrentScreen` enum でアクティブなスクリーンを保持する。

```rust
pub enum CurrentScreen {
    Main(MainScreen),
    Events(EventsScreen),
    Viewer(ViewerScreen),
    EventSearch(EventSearchScreen),
    GroupEvents(GroupEventsScreen),
}
```

**理由**: Rust の enum は1つのバリアントのみがアクティブであることを型レベルで保証できる。不要なスクリーンの状態がメモリに残らない。`handle_key` と `draw` のディスパッチが自然に書ける。

**代替案**: 全スクリーンを `App` のフィールドとして常時保持する案 → スクリーン遷移時に不要な状態が残り、初期化忘れのバグが起きやすいため不採用。ただし、`MainScreen` の `log_groups` のようにスクリーン間で再利用が必要なデータはスクリーン遷移時に引き渡す設計とする。

### 5. UI描画関数のシグネチャ変更

**決定**: 各 `ui::*_screen::draw` のシグネチャを `&App` から各スクリーン構造体の参照に変更する。

```rust
// Before
pub fn draw(f: &mut Frame, app: &mut App)

// After
pub fn draw(f: &mut Frame, screen: &MainScreen)
```

`ui::draw` のトップレベル関数は `CurrentScreen` に対してマッチして各描画関数にディスパッチする。

## Risks / Trade-offs

**[スクリーン間のデータ受け渡しの複雑化]** → `NavigateTo` enum に遷移時の必要データを明示的に含めることで管理。`App::run` でのディスパッチ処理は多少増えるが、各スクリーンの独立性が向上するトレードオフとして許容。

**[MainScreen の log_groups を他スクリーンが参照する問題]** → EventSearch/Events への遷移時に必要なグループ名・ストリーム名は `NavigateTo` に含めて渡す。遷移元のスクリーンは消費されるので、必要なデータは `NavigateTo` にムーブする。

**[log_events が Events と GroupEvents で共用されている問題]** → 各スクリーンが独立した `StatefulList<LogEvent>` を持つ設計に変更。データの重複は許容し、スクリーン遷移時に新規ロードする。

**[リファクタリング範囲の大きさ]** → `app.rs` と `ui/` の全ファイルに影響する。段階的に進めるため、まず構造体の分離を行い、次にメソッドの移動、最後にUIシグネチャ変更の順で実施する。
