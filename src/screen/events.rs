//! ログイベント一覧スクリーンの状態管理。
//!
//! 指定したログストリームのイベントを一覧表示し、
//! フィルタリングと JSONL ダウンロードをサポートします。

use anyhow::Result;
use aws_sdk_cloudwatchlogs::Client;
use crossterm::event::KeyCode;
use std::sync::Arc;

use super::{CurrentScreen, NavigateTo, ScreenAction};
use crate::app::{LogEvent, StatefulList};
use crate::aws;

/// ログイベント一覧を表示するスクリーン。
///
/// `j`/`k` でリスト移動、`Enter` で詳細表示、`/` でフィルタ編集、
/// `d` でダウンロードパス入力、`q` で前の画面に戻ります。
pub struct EventsScreen {
    /// 共有 AWS CloudWatch Logs クライアント
    pub client: Arc<Client>,
    /// ログイベントのリスト状態
    pub log_events: StatefulList<LogEvent>,
    /// 現在適用中のフィルタパターン（`None` は全件表示）
    pub filter_input: Option<String>,
    /// フィルタ入力モードがアクティブかどうか
    pub filter_editing: bool,
    /// フィルタ入力バッファ
    pub filter_buffer: String,
    /// ダウンロードパス入力モードがアクティブかどうか
    pub download_editing: bool,
    /// ダウンロードパス入力バッファ
    pub download_path_buffer: String,
    /// ダウンロード結果メッセージ（成功またはエラー）
    pub download_status: Option<String>,
    /// 対象のロググループ名
    pub group_name: String,
    /// 対象のログストリーム名
    pub stream_name: String,
    /// 前の画面（`q` で戻るため保持）
    pub origin: Option<Box<CurrentScreen>>,
}

impl EventsScreen {
    /// 新しい [`EventsScreen`] を生成します。
    ///
    /// `origin` には前の画面を渡し、`q` キーで戻れるようにします。
    pub fn new(
        client: Arc<Client>,
        group_name: String,
        stream_name: String,
        origin: Box<CurrentScreen>,
    ) -> Self {
        Self {
            client,
            log_events: StatefulList::new(),
            filter_input: None,
            filter_editing: false,
            filter_buffer: String::new(),
            download_editing: false,
            download_path_buffer: String::new(),
            download_status: None,
            group_name,
            stream_name,
            origin: Some(origin),
        }
    }

    /// キー入力を処理して [`ScreenAction`] を返します。
    ///
    /// ダウンロードパス入力モードとフィルタ入力モードを優先的に処理します。
    pub async fn handle_key(&mut self, code: KeyCode) -> Result<ScreenAction> {
        self.download_status = None;
        if self.download_editing {
            match code {
                KeyCode::Enter => {
                    let path = self.download_path_buffer.clone();
                    self.write_events_to_jsonl(&path);
                    self.download_editing = false;
                }
                KeyCode::Esc => {
                    self.download_editing = false;
                    self.download_path_buffer.clear();
                }
                KeyCode::Backspace => {
                    self.download_path_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.download_path_buffer.push(c);
                }
                _ => {}
            }
            return Ok(ScreenAction::None);
        }
        if self.filter_editing {
            match code {
                KeyCode::Enter => {
                    let pattern = self.filter_buffer.clone();
                    self.filter_input = if pattern.is_empty() {
                        None
                    } else {
                        Some(pattern)
                    };
                    self.filter_editing = false;
                    self.log_events = StatefulList::new();
                    self.load_log_events().await?;
                }
                KeyCode::Esc => {
                    self.filter_editing = false;
                    self.filter_buffer.clear();
                }
                KeyCode::Backspace => {
                    self.filter_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.filter_buffer.push(c);
                }
                _ => {}
            }
            return Ok(ScreenAction::None);
        }
        match code {
            KeyCode::Char('q') => {
                if let Some(origin) = self.origin.take() {
                    return Ok(ScreenAction::Navigate(NavigateTo::Restore(origin)));
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.log_events.next(),
            KeyCode::Char('k') | KeyCode::Up => self.log_events.previous(),
            KeyCode::Char('/') => {
                self.filter_editing = true;
                self.filter_buffer = self.filter_input.clone().unwrap_or_default();
            }
            KeyCode::Char('d') => {
                self.download_path_buffer = self.default_download_path();
                self.download_editing = true;
            }
            KeyCode::Enter => {
                if let Some(event) = self.log_events.selected().cloned() {
                    return Ok(ScreenAction::Navigate(NavigateTo::NewViewer { event }));
                }
            }
            _ => {}
        }
        Ok(ScreenAction::None)
    }

    /// カーソルが末尾付近に達した場合にページネーションで追加ロードします。
    ///
    /// メインループ毎フレームで呼び出されます。
    pub async fn check_pagination(&mut self) -> Result<()> {
        if let Some(idx) = self.log_events.selected_index() {
            let len = self.log_events.items.len();
            if len > 0
                && idx + 5 >= len
                && self.log_events.next_token.is_some()
                && !self.log_events.loading
            {
                self.load_more_events().await?;
            }
        }
        Ok(())
    }

    /// ログイベントを初回ロードします（現在のフィルタを適用）。
    pub async fn load_log_events(&mut self) -> Result<()> {
        let filter = self.filter_input.clone();
        self.log_events.loading = true;
        let (events, token) = aws::fetch_log_events(
            &self.client,
            &self.group_name,
            Some(&self.stream_name),
            None,
            None,
            filter,
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

    async fn load_more_events(&mut self) -> Result<()> {
        let filter = self.filter_input.clone();
        let token = self.log_events.next_token.clone();
        self.log_events.loading = true;
        let (events, next) = aws::fetch_log_events(
            &self.client,
            &self.group_name,
            Some(&self.stream_name),
            None,
            None,
            filter,
            token,
        )
        .await?;
        self.log_events.items.extend(events);
        self.log_events.next_token = next;
        self.log_events.loading = false;
        Ok(())
    }

    fn default_download_path(&self) -> String {
        let group_short = self
            .group_name
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or("unknown");
        let date = jiff::Zoned::now().date();
        format!("{}-{}.jsonl", group_short, date)
    }

    fn write_events_to_jsonl(&mut self, path: &str) {
        let lines: Vec<String> = self
            .log_events
            .items
            .iter()
            .map(|e| {
                serde_json::json!({
                    "timestamp": e.timestamp,
                    "message": e.message,
                })
                .to_string()
            })
            .collect();
        let content = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
        match std::fs::write(path, content) {
            Ok(_) => self.download_status = Some(format!("Saved: {}", path)),
            Err(e) => self.download_status = Some(format!("Error: {}", e)),
        }
    }
}
