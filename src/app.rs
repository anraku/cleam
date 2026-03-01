//! アプリケーションのコアデータ構造とメインループを定義するモジュール。
//!
//! [`App`] がアプリケーション全体のエントリポイントであり、
//! [`StatefulList`] がリスト状態管理のジェネリック型です。

use anyhow::Result;
use aws_sdk_cloudwatchlogs::Client;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::sync::Arc;
use std::time::Duration;

use crate::screen::event_search::EventSearchScreen;
use crate::screen::{
    CurrentScreen, EventsScreen, GroupEventsScreen, MainScreen, NavigateTo, ScreenAction,
    ViewerScreen,
};
use crate::ui;

/// ページネーションと絞り込みに対応したリストの状態管理構造体。
///
/// `T` はリストアイテムの型です。[`ratatui::widgets::ListState`] を内包し、
/// TUI ウィジェットへの選択状態の受け渡しを行います。
pub struct StatefulList<T> {
    /// リストアイテムの全件データ
    pub items: Vec<T>,
    /// ratatui の選択状態
    pub state: ratatui::widgets::ListState,
    /// AWS API のページネーショントークン
    pub next_token: Option<String>,
    /// 追加ロード中フラグ
    pub loading: bool,
    /// 絞り込み時に表示するアイテムのインデックス一覧（`None` は全件表示）
    pub visible_indices: Option<Vec<usize>>,
}

impl<T> StatefulList<T> {
    /// 空の [`StatefulList`] を生成します。
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ratatui::widgets::ListState::default(),
            next_token: None,
            loading: false,
            visible_indices: None,
        }
    }

    /// 選択を次のアイテムに移動します。末尾では移動しません。
    pub fn next(&mut self) {
        let len = match &self.visible_indices {
            Some(v) => v.len(),
            None => self.items.len(),
        };
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len.saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// 選択を前のアイテムに移動します。先頭では移動しません。
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// 現在選択中のアイテムへの参照を返します。
    ///
    /// `visible_indices` が設定されている場合は絞り込み後のインデックスを使用します。
    pub fn selected(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|i| match &self.visible_indices {
                Some(v) => v.get(i).and_then(|&orig| self.items.get(orig)),
                None => self.items.get(i),
            })
    }

    /// 現在選択中のアイテムの `items` 内インデックスを返します。
    ///
    /// 絞り込み中は `visible_indices` を経由した元のインデックスを返します。
    pub fn selected_index(&self) -> Option<usize> {
        self.state
            .selected()
            .and_then(|i| match &self.visible_indices {
                Some(v) => v.get(i).copied(),
                None => Some(i),
            })
    }

    /// 現在表示対象のアイテム一覧を返します。
    ///
    /// `visible_indices` が設定されている場合は絞り込み後のアイテムのみを返します。
    pub fn visible_items(&self) -> Vec<&T> {
        match &self.visible_indices {
            Some(v) => v.iter().filter_map(|&i| self.items.get(i)).collect(),
            None => self.items.iter().collect(),
        }
    }
}

/// メインスクリーンでフォーカスされているパネルを示す列挙型。
#[derive(Debug, Clone, PartialEq)]
pub enum ActivePanel {
    /// ロググループ一覧パネル
    Groups,
    /// ログストリーム一覧パネル
    Streams,
}

/// CloudWatch Logs のロググループを表す構造体。
#[derive(Debug, Clone)]
pub struct LogGroup {
    /// ロググループ名
    pub name: String,
}

/// CloudWatch Logs のログストリームを表す構造体。
#[derive(Debug, Clone)]
pub struct LogStream {
    /// ログストリーム名
    pub name: String,
    /// 最終イベントのタイムスタンプ（Unix ミリ秒）
    pub last_event_time: Option<i64>,
}

/// CloudWatch Logs の個別ログイベントを表す構造体。
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// イベントのタイムスタンプ（Unix ミリ秒）
    pub timestamp: i64,
    /// ログメッセージ本文
    pub message: String,
}

/// アプリケーション全体の状態を管理する構造体。
///
/// AWS クライアントと現在表示中の画面を保持し、
/// キーイベントの処理と画面遷移を制御します。
pub struct App {
    /// 共有 AWS CloudWatch Logs クライアント
    pub client: Arc<Client>,
    /// 現在アクティブな画面
    pub screen: CurrentScreen,
    /// 次のフレーム描画前にターミナルをクリアするフラグ
    pub needs_clear: bool,
}

impl App {
    /// AWS クライアントを受け取り、メインスクリーンの初期状態で [`App`] を生成します。
    pub fn new(client: Client) -> Self {
        let client = Arc::new(client);
        let main_screen = MainScreen::new(Arc::clone(&client));
        Self {
            client,
            screen: CurrentScreen::Main(main_screen),
            needs_clear: false,
        }
    }

    /// アプリケーションのメインループを実行します。
    ///
    /// ロググループの初回ロード後、キーイベントを待機して画面の更新と
    /// 画面遷移を繰り返します。`q` キーで終了します。
    ///
    /// # Errors
    ///
    /// - AWS API 呼び出しやターミナル操作に失敗した場合
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        if let CurrentScreen::Main(s) = &mut self.screen {
            s.load_log_groups().await?;
        }

        loop {
            if self.needs_clear {
                terminal.clear()?;
                self.needs_clear = false;
            }
            terminal.draw(|f| ui::draw(f, &mut self.screen))?;

            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let action = match &mut self.screen {
                    CurrentScreen::Main(s) => s.handle_key(key.code).await?,
                    CurrentScreen::Events(s) => s.handle_key(key.code).await?,
                    CurrentScreen::Viewer(s) => s.handle_key(key.code).await?,
                    CurrentScreen::EventSearch(s) => s.handle_key(key.code).await?,
                    CurrentScreen::GroupEvents(s) => s.handle_key(key.code).await?,
                    CurrentScreen::Transitioning => ScreenAction::None,
                };
                match action {
                    ScreenAction::None => {}
                    ScreenAction::Quit => break,
                    ScreenAction::Navigate(nav) => {
                        self.needs_clear = true;
                        self.handle_navigate(nav).await?;
                    }
                }
            }

            if let CurrentScreen::Main(s) = &mut self.screen {
                s.check_group_change().await?;
                s.check_pagination().await?;
            }
            if let CurrentScreen::Events(s) = &mut self.screen {
                s.check_pagination().await?;
            }
        }

        Ok(())
    }

    async fn handle_navigate(&mut self, nav: NavigateTo) -> Result<()> {
        match nav {
            NavigateTo::NewEvents {
                group_name,
                stream_name,
            } => {
                let origin = std::mem::replace(&mut self.screen, CurrentScreen::Transitioning);
                let mut s = EventsScreen::new(
                    Arc::clone(&self.client),
                    group_name,
                    stream_name,
                    Box::new(origin),
                );
                s.load_log_events().await?;
                self.screen = CurrentScreen::Events(s);
            }
            NavigateTo::NewViewer { event } => {
                let origin = std::mem::replace(&mut self.screen, CurrentScreen::Transitioning);
                let viewer = ViewerScreen::new(event, Box::new(origin));
                self.screen = CurrentScreen::Viewer(viewer);
            }
            NavigateTo::NewEventSearch { group_name } => {
                let origin = std::mem::replace(&mut self.screen, CurrentScreen::Transitioning);
                let s = EventSearchScreen::new(group_name, Box::new(origin));
                self.screen = CurrentScreen::EventSearch(s);
            }
            NavigateTo::NewGroupEvents {
                group_name,
                start_ms,
                end_ms,
                pattern,
                start_display,
                end_display,
                pattern_display,
            } => {
                let origin = std::mem::replace(&mut self.screen, CurrentScreen::Transitioning);
                let mut s = GroupEventsScreen::new(
                    Arc::clone(&self.client),
                    group_name,
                    start_display,
                    end_display,
                    pattern_display,
                    Box::new(origin),
                );
                s.load_group_events(start_ms, end_ms, pattern).await?;
                self.screen = CurrentScreen::GroupEvents(s);
            }
            NavigateTo::Restore(screen) => {
                self.screen = *screen;
            }
        }
        Ok(())
    }
}
