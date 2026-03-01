//! ログイベント詳細ビューアスクリーンの状態管理。
//!
//! 選択されたログイベントのメッセージ全文をスクロール表示します。

use anyhow::Result;
use crossterm::event::KeyCode;

use super::{CurrentScreen, NavigateTo, ScreenAction};
use crate::app::LogEvent;

/// ログイベントの詳細を全画面表示するスクリーン。
///
/// `j`/`k` でスクロール、`q` で前の画面に戻ります。
pub struct ViewerScreen {
    /// 表示対象のログイベント
    pub selected_event: LogEvent,
    /// 現在のスクロールオフセット（行数）
    pub viewer_scroll: u16,
    /// 前の画面（`q` で戻るため保持）
    pub origin: Option<Box<CurrentScreen>>,
}

impl ViewerScreen {
    /// ログイベントと前の画面を受け取り、[`ViewerScreen`] を生成します。
    pub fn new(event: LogEvent, origin: Box<CurrentScreen>) -> Self {
        Self {
            selected_event: event,
            viewer_scroll: 0,
            origin: Some(origin),
        }
    }

    /// キー入力を処理して [`ScreenAction`] を返します。
    ///
    /// `j`/`Down` でスクロールダウン、`k`/`Up` でスクロールアップ、
    /// `q` で前の画面に戻ります。
    pub async fn handle_key(&mut self, code: KeyCode) -> Result<ScreenAction> {
        match code {
            KeyCode::Char('q') => {
                if let Some(origin) = self.origin.take() {
                    return Ok(ScreenAction::Navigate(NavigateTo::Restore(origin)));
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.viewer_scroll = self.viewer_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.viewer_scroll = self.viewer_scroll.saturating_sub(1);
            }
            _ => {}
        }
        Ok(ScreenAction::None)
    }
}
