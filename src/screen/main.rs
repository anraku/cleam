//! メインスクリーンの状態管理。
//!
//! ロググループとログストリームの二ペイン表示を管理し、
//! キーボードナビゲーションとインクリメンタル検索を処理します。

use anyhow::Result;
use aws_sdk_cloudwatchlogs::Client;
use crossterm::event::KeyCode;
use std::sync::Arc;

use super::{NavigateTo, ScreenAction};
use crate::app::{ActivePanel, LogGroup, LogStream, StatefulList};
use crate::aws;

/// ロググループとログストリームを表示するメインスクリーン。
///
/// `h`/`l` でパネル切替、`j`/`k` でリスト移動、`/` で検索、
/// `Enter` でイベント一覧へ遷移、`g` でイベント検索フォームへ遷移します。
pub struct MainScreen {
    /// 共有 AWS CloudWatch Logs クライアント
    pub client: Arc<Client>,
    /// ロググループのリスト状態
    pub log_groups: StatefulList<LogGroup>,
    /// ログストリームのリスト状態
    pub log_streams: StatefulList<LogStream>,
    /// 現在フォーカスされているパネル
    pub active_panel: ActivePanel,
    /// インクリメンタル検索の入力バッファ
    pub main_search_query: String,
    /// 検索モードがアクティブかどうか
    pub main_search_active: bool,
    /// ストリームリロードのトリガー検出用の前回選択グループインデックス
    pub last_selected_group: Option<usize>,
}

impl MainScreen {
    /// AWS クライアントを受け取り、初期状態の [`MainScreen`] を生成します。
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            log_groups: StatefulList::new(),
            log_streams: StatefulList::new(),
            active_panel: ActivePanel::Groups,
            main_search_query: String::new(),
            main_search_active: false,
            last_selected_group: None,
        }
    }

    /// キー入力を処理して [`ScreenAction`] を返します。
    ///
    /// 検索モード中は文字入力・バックスペース・Esc・Enter のみを受け付けます。
    /// 通常モードでは vim ライクなキーバインドで操作します。
    pub async fn handle_key(&mut self, code: KeyCode) -> Result<ScreenAction> {
        if self.main_search_active {
            match code {
                KeyCode::Esc => {
                    self.clear_main_search();
                }
                KeyCode::Backspace => {
                    self.main_search_query.pop();
                    self.apply_main_search();
                }
                KeyCode::Enter => {
                    if self.main_search_query.is_empty() {
                        self.clear_main_search();
                    } else {
                        self.main_search_active = false;
                        self.main_search_query.clear();
                    }
                }
                KeyCode::Char(c) => {
                    self.main_search_query.push(c);
                    self.apply_main_search();
                }
                _ => {}
            }
            return Ok(ScreenAction::None);
        }

        match code {
            KeyCode::Char('q') => return Ok(ScreenAction::Quit),
            KeyCode::Char('l') => {
                self.active_panel = ActivePanel::Streams;
            }
            KeyCode::Char('h') => {
                self.active_panel = ActivePanel::Groups;
            }
            KeyCode::Char('j') | KeyCode::Down => match self.active_panel {
                ActivePanel::Groups => self.log_groups.next(),
                ActivePanel::Streams => self.log_streams.next(),
            },
            KeyCode::Char('k') | KeyCode::Up => match self.active_panel {
                ActivePanel::Groups => self.log_groups.previous(),
                ActivePanel::Streams => self.log_streams.previous(),
            },
            KeyCode::Char('/') => {
                self.main_search_active = true;
            }
            KeyCode::Char('g') => {
                if self.log_groups.state.selected().is_some() && !self.log_groups.items.is_empty() {
                    let group_name = self
                        .log_groups
                        .selected()
                        .map(|g| g.name.clone())
                        .unwrap_or_default();
                    return Ok(ScreenAction::Navigate(NavigateTo::NewEventSearch {
                        group_name,
                    }));
                }
            }
            KeyCode::Enter => {
                if self.active_panel == ActivePanel::Streams
                    && !self.log_streams.items.is_empty()
                    && self.log_streams.state.selected().is_some()
                {
                    let group_name = self
                        .log_groups
                        .selected()
                        .map(|g| g.name.clone())
                        .unwrap_or_default();
                    let stream_name = self
                        .log_streams
                        .selected()
                        .map(|s| s.name.clone())
                        .unwrap_or_default();
                    return Ok(ScreenAction::Navigate(NavigateTo::NewEvents {
                        group_name,
                        stream_name,
                    }));
                } else if self.active_panel == ActivePanel::Groups {
                    self.active_panel = ActivePanel::Streams;
                }
            }
            _ => {}
        }
        Ok(ScreenAction::None)
    }

    /// 検索クエリを元にアクティブパネルのリストを絞り込みます。
    ///
    /// クエリが空の場合は `visible_indices` を `None` にリセットします。
    pub fn apply_main_search(&mut self) {
        let query = self.main_search_query.to_lowercase();
        match self.active_panel {
            ActivePanel::Groups => {
                if query.is_empty() {
                    self.log_groups.visible_indices = None;
                } else {
                    let indices: Vec<usize> = self
                        .log_groups
                        .items
                        .iter()
                        .enumerate()
                        .filter(|(_, g)| g.name.to_lowercase().contains(&query))
                        .map(|(i, _)| i)
                        .collect();
                    let new_pos = if indices.is_empty() { None } else { Some(0) };
                    self.log_groups.visible_indices = Some(indices);
                    self.log_groups.state.select(new_pos);
                }
            }
            ActivePanel::Streams => {
                if query.is_empty() {
                    self.log_streams.visible_indices = None;
                } else {
                    let indices: Vec<usize> = self
                        .log_streams
                        .items
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| s.name.to_lowercase().contains(&query))
                        .map(|(i, _)| i)
                        .collect();
                    let new_pos = if indices.is_empty() { None } else { Some(0) };
                    self.log_streams.visible_indices = Some(indices);
                    self.log_streams.state.select(new_pos);
                }
            }
        }
    }

    /// 検索状態をクリアし、絞り込み前の選択位置を復元します。
    pub fn clear_main_search(&mut self) {
        let selected_group = self.log_groups.selected_index();
        let selected_stream = self.log_streams.selected_index();
        self.main_search_active = false;
        self.main_search_query.clear();
        self.log_groups.visible_indices = None;
        self.log_streams.visible_indices = None;
        self.log_groups.state.select(selected_group);
        self.log_streams.state.select(selected_stream);
    }

    /// 選択グループが変わった場合にストリームリストを再ロードします。
    ///
    /// メインループ毎フレームで呼び出されます。
    pub async fn check_group_change(&mut self) -> Result<()> {
        let current = self.log_groups.selected_index();
        if current != self.last_selected_group {
            self.last_selected_group = current;
            self.log_streams = StatefulList::new();
            if current.is_some() {
                self.load_log_streams().await?;
            }
        }
        Ok(())
    }

    /// カーソルが末尾付近に達した場合にページネーションで追加ロードします。
    ///
    /// メインループ毎フレームで呼び出されます。
    pub async fn check_pagination(&mut self) -> Result<()> {
        if let Some(idx) = self.log_groups.selected_index() {
            let len = self.log_groups.items.len();
            if len > 0
                && idx + 5 >= len
                && self.log_groups.next_token.is_some()
                && !self.log_groups.loading
            {
                self.load_more_groups().await?;
            }
        }
        if self.active_panel == ActivePanel::Streams
            && let Some(idx) = self.log_streams.selected_index()
        {
            let len = self.log_streams.items.len();
            if len > 0
                && idx + 5 >= len
                && self.log_streams.next_token.is_some()
                && !self.log_streams.loading
            {
                self.load_more_streams().await?;
            }
        }
        Ok(())
    }

    /// ロググループを初回ロードします（ページ先頭から取得）。
    pub async fn load_log_groups(&mut self) -> Result<()> {
        self.log_groups.loading = true;
        let (groups, token) = aws::fetch_log_groups(&self.client, None).await?;
        self.log_groups.items = groups;
        self.log_groups.next_token = token;
        self.log_groups.loading = false;
        if !self.log_groups.items.is_empty() {
            self.log_groups.state.select(Some(0));
        }
        Ok(())
    }

    async fn load_more_groups(&mut self) -> Result<()> {
        self.log_groups.loading = true;
        let token = self.log_groups.next_token.clone();
        let (groups, next) = aws::fetch_log_groups(&self.client, token).await?;
        self.log_groups.items.extend(groups);
        self.log_groups.next_token = next;
        self.log_groups.loading = false;
        if self.active_panel == ActivePanel::Groups && !self.main_search_query.is_empty() {
            self.apply_main_search();
        }
        Ok(())
    }

    /// 現在選択中のロググループのログストリームを初回ロードします。
    pub async fn load_log_streams(&mut self) -> Result<()> {
        let group_name = match self.log_groups.selected() {
            Some(g) => g.name.clone(),
            None => return Ok(()),
        };
        self.log_streams.loading = true;
        let (streams, token) = aws::fetch_log_streams(&self.client, &group_name, None).await?;
        self.log_streams.items = streams;
        self.log_streams.next_token = token;
        self.log_streams.loading = false;
        if !self.log_streams.items.is_empty() {
            self.log_streams.state.select(Some(0));
        }
        Ok(())
    }

    async fn load_more_streams(&mut self) -> Result<()> {
        let group_name = match self.log_groups.selected() {
            Some(g) => g.name.clone(),
            None => return Ok(()),
        };
        self.log_streams.loading = true;
        let token = self.log_streams.next_token.clone();
        let (streams, next) = aws::fetch_log_streams(&self.client, &group_name, token).await?;
        self.log_streams.items.extend(streams);
        self.log_streams.next_token = next;
        self.log_streams.loading = false;
        if !self.main_search_query.is_empty() {
            self.apply_main_search();
        }
        Ok(())
    }
}
