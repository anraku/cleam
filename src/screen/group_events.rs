//! グループ横断イベント一覧スクリーンの状態管理。
//!
//! [`EventSearchScreen`] で指定した条件でロググループ全体を検索した結果を表示します。
//!
//! [`EventSearchScreen`]: crate::screen::event_search::EventSearchScreen

use anyhow::Result;
use aws_sdk_cloudwatchlogs::Client;
use crossterm::event::KeyCode;
use std::sync::Arc;

use super::{CurrentScreen, NavigateTo, ScreenAction};
use crate::app::{LogEvent, StatefulList};
use crate::aws;

/// ロググループ全体を横断して検索したイベント一覧を表示するスクリーン。
///
/// `j`/`k` でリスト移動、`Enter` で詳細表示、`q` で前の画面に戻ります。
pub struct GroupEventsScreen {
    /// 共有 AWS CloudWatch Logs クライアント
    pub client: Arc<Client>,
    /// 検索結果のログイベントリスト状態
    pub log_events: StatefulList<LogEvent>,
    /// 検索対象のロググループ名
    pub group_name: String,
    /// UI 表示用の検索開始時刻文字列
    pub start_display: String,
    /// UI 表示用の検索終了時刻文字列
    pub end_display: String,
    /// UI 表示用のフィルタパターン文字列
    pub pattern_display: String,
    /// 前の画面（`q` で戻るため保持）
    pub origin: Option<Box<CurrentScreen>>,
}

impl GroupEventsScreen {
    /// 新しい [`GroupEventsScreen`] を生成します。
    ///
    /// `start_display`・`end_display`・`pattern_display` は UI 表示用の文字列です。
    pub fn new(
        client: Arc<Client>,
        group_name: String,
        start_display: String,
        end_display: String,
        pattern_display: String,
        origin: Box<CurrentScreen>,
    ) -> Self {
        Self {
            client,
            log_events: StatefulList::new(),
            group_name,
            start_display,
            end_display,
            pattern_display,
            origin: Some(origin),
        }
    }

    /// キー入力を処理して [`ScreenAction`] を返します。
    pub async fn handle_key(&mut self, code: KeyCode) -> Result<ScreenAction> {
        match code {
            KeyCode::Char('q') => {
                if let Some(origin) = self.origin.take() {
                    return Ok(ScreenAction::Navigate(NavigateTo::Restore(origin)));
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.log_events.next(),
            KeyCode::Char('k') | KeyCode::Up => self.log_events.previous(),
            KeyCode::Enter => {
                if let Some(event) = self.log_events.selected().cloned() {
                    return Ok(ScreenAction::Navigate(NavigateTo::NewViewer { event }));
                }
            }
            _ => {}
        }
        Ok(ScreenAction::None)
    }

    /// 指定した時間範囲とフィルタパターンでログイベントをロードします。
    ///
    /// # Arguments
    ///
    /// * `start_ms` - 検索開始時刻（Unix ミリ秒、`None` で無制限）
    /// * `end_ms` - 検索終了時刻（Unix ミリ秒、`None` で無制限）
    /// * `pattern` - CloudWatch Logs フィルタパターン（`None` で全件）
    pub async fn load_group_events(
        &mut self,
        start_ms: Option<i64>,
        end_ms: Option<i64>,
        pattern: Option<String>,
    ) -> Result<()> {
        self.log_events.loading = true;
        let (events, token) = aws::fetch_log_events(
            &self.client,
            &self.group_name,
            None,
            start_ms,
            end_ms,
            pattern,
            None,
        )
        .await?;
        self.log_events.items = events;
        self.log_events.next_token = token;
        self.log_events.loading = false;
        if !self.log_events.items.is_empty() {
            self.log_events.state.select(Some(0));
        }
        Ok(())
    }
}
