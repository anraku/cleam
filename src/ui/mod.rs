//! TUI 描画ロジックを提供するモジュール。
//!
//! 各スクリーンに対応したサブモジュールと、現在の画面を描画するエントリポイント
//! [`draw`] を公開します。

mod event_search_screen;
mod events_screen;
mod group_events_screen;
mod main_screen;
mod viewer_screen;

use ratatui::Frame;

use crate::screen::CurrentScreen;

/// 現在の画面に対応する描画関数を呼び出します。
///
/// [`ratatui::Terminal::draw`] のコールバックから呼び出されます。
pub fn draw(f: &mut Frame, screen: &mut CurrentScreen) {
    match screen {
        CurrentScreen::Main(s) => main_screen::draw(f, s),
        CurrentScreen::Events(s) => events_screen::draw(f, s),
        CurrentScreen::Viewer(s) => viewer_screen::draw(f, s),
        CurrentScreen::EventSearch(s) => event_search_screen::draw(f, s),
        CurrentScreen::GroupEvents(s) => group_events_screen::draw(f, s),
        CurrentScreen::Transitioning => {}
    }
}
