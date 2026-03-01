//! # cleam
//!
//! `cleam` は AWS CloudWatch Logs をターミナル上でインタラクティブに閲覧するための TUI アプリケーションです。
//!
//! ## 主な機能
//!
//! - ロググループ・ログストリームの一覧表示とキーボードナビゲーション
//! - ログイベントの閲覧・絞り込み・JSONL ダウンロード
//! - 時間範囲とフィルタパターンによるクロスストリーム検索
//! - vim ライクなキーバインド (`j`/`k` で移動、`q` で戻る)

mod app;
mod aws;
mod screen;
mod tui;
mod ui;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let client = aws::build_client().await?;
    let mut app = App::new(client);
    let mut terminal = tui::init()?;
    let result = app.run(&mut terminal).await;
    tui::restore()?;
    result
}
