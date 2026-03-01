//! 各画面の状態管理と画面遷移の定義。
//!
//! [`CurrentScreen`] が現在の画面を保持し、
//! [`ScreenAction`] を通じてメインループへ操作を通知します。

pub mod event_search;
pub mod events;
pub mod group_events;
pub mod main;
pub mod viewer;

pub use event_search::EventSearchScreen;
pub use events::EventsScreen;
pub use group_events::GroupEventsScreen;
pub use main::MainScreen;
pub use viewer::ViewerScreen;

use crate::app::LogEvent;

/// 各画面のキーハンドラが返すアクション。
pub enum ScreenAction {
    /// 何もしない
    None,
    /// アプリケーションを終了する
    Quit,
    /// 指定した画面に遷移する
    Navigate(NavigateTo),
}

/// [`ScreenAction::Navigate`] が指定する遷移先。
pub enum NavigateTo {
    /// 指定ストリームのイベント一覧画面へ遷移する
    NewEvents {
        /// 対象のロググループ名
        group_name: String,
        /// 対象のログストリーム名
        stream_name: String,
    },
    /// ログイベント詳細ビューア画面へ遷移する
    NewViewer {
        /// 表示するログイベント
        event: LogEvent,
    },
    /// イベント検索フォーム画面へ遷移する
    NewEventSearch {
        /// 検索対象のロググループ名
        group_name: String,
    },
    /// グループ横断イベント一覧画面へ遷移する
    NewGroupEvents {
        /// 検索対象のロググループ名
        group_name: String,
        /// 検索開始時刻（Unix ミリ秒）
        start_ms: Option<i64>,
        /// 検索終了時刻（Unix ミリ秒）
        end_ms: Option<i64>,
        /// CloudWatch Logs フィルタパターン
        pattern: Option<String>,
        /// UI 表示用の開始時刻文字列
        start_display: String,
        /// UI 表示用の終了時刻文字列
        end_display: String,
        /// UI 表示用のパターン文字列
        pattern_display: String,
    },
    /// 元の画面に戻る
    Restore(Box<CurrentScreen>),
}

/// 現在アクティブな画面を保持する列挙型。
pub enum CurrentScreen {
    /// メインスクリーン（ロググループ・ストリーム一覧）
    Main(MainScreen),
    /// ログイベント一覧スクリーン
    Events(EventsScreen),
    /// ログイベント詳細ビューアスクリーン
    Viewer(ViewerScreen),
    /// イベント検索フォームスクリーン
    EventSearch(EventSearchScreen),
    /// グループ横断イベント一覧スクリーン
    GroupEvents(GroupEventsScreen),
    /// 画面遷移中の一時状態（`mem::replace` で使用）
    Transitioning,
}
