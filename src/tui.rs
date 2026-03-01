//! ターミナルの初期化・復元ユーティリティ。
//!
//! [`ratatui::init`] / [`ratatui::restore`] をラップし、Raw モードと
//! オルタネートスクリーンの切り替えを行います。

use anyhow::Result;
use ratatui::DefaultTerminal;

/// ターミナルを TUI モードで初期化して返します。
///
/// 内部で `enable_raw_mode` と `EnterAlternateScreen` を実行します。
pub fn init() -> Result<DefaultTerminal> {
    // ratatui::init() handles enable_raw_mode + EnterAlternateScreen internally
    Ok(ratatui::init())
}

/// ターミナルを通常モードに復元します。
///
/// 内部で `disable_raw_mode` と `LeaveAlternateScreen` を実行します。
pub fn restore() -> Result<()> {
    // ratatui::restore() handles disable_raw_mode + LeaveAlternateScreen internally
    ratatui::restore();
    Ok(())
}
