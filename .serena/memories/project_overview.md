# cleam プロジェクト概要

## 目的
AWS CloudWatch Logs を TUI でインタラクティブに閲覧・検索するツール。

## 技術スタック
- Rust (edition 2024)
- ratatui 0.29 (TUI フレームワーク)
- crossterm 0.28 (ターミナル操作)
- aws-sdk-cloudwatchlogs 1.x
- tokio (async ランタイム)
- jiff (日時処理)
- serde_json (JSON シリアライズ)
- anyhow (エラーハンドリング)

## ディレクトリ構成
```
src/
  main.rs              - エントリポイント
  app.rs               - コアデータ構造 (App, StatefulList, LogGroup/Stream/Event, ActivePanel)
  aws.rs               - AWS CloudWatch API ラッパー
  tui.rs               - ターミナル初期化/復元
  screen/
    mod.rs             - ScreenAction, NavigateTo, CurrentScreen 列挙型
    main.rs            - MainScreen (ロググループ・ストリーム二ペイン)
    events.rs          - EventsScreen (ストリームイベント一覧)
    viewer.rs          - ViewerScreen (イベント詳細)
    event_search.rs    - EventSearchScreen (時間範囲・パターン検索フォーム)
    group_events.rs    - GroupEventsScreen (グループ横断検索結果)
  ui/
    mod.rs             - draw() エントリポイント
    main_screen.rs     - MainScreen の描画
    events_screen.rs   - EventsScreen の描画
    viewer_screen.rs   - ViewerScreen の描画
    event_search_screen.rs - EventSearchScreen の描画
    group_events_screen.rs - GroupEventsScreen の描画
```
