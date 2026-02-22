use anyhow::Result;
use aws_sdk_cloudwatchlogs::Client;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::time::Duration;

use crate::aws;
use crate::ui;

pub struct StatefulList<T> {
    pub items: Vec<T>,
    pub state: ratatui::widgets::ListState,
    pub next_token: Option<String>,
    pub loading: bool,
}

impl<T> StatefulList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ratatui::widgets::ListState::default(),
            next_token: None,
            loading: false,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Main,
    Events,
    Viewer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivePanel {
    Groups,
    Streams,
}

#[derive(Debug, Clone)]
pub struct LogGroup {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LogStream {
    pub name: String,
    pub last_event_time: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub timestamp: i64,
    pub message: String,
}

pub struct App {
    pub screen: Screen,
    pub active_panel: ActivePanel,
    pub client: Client,
    pub log_groups: StatefulList<LogGroup>,
    pub log_streams: StatefulList<LogStream>,
    pub log_events: StatefulList<LogEvent>,
    pub selected_event: Option<LogEvent>,
    pub filter_input: Option<String>,
    pub filter_editing: bool,
    pub filter_buffer: String,
    pub viewer_scroll: u16,
    pub last_selected_group: Option<usize>,
    pub needs_clear: bool,
}

impl App {
    pub fn new(client: Client) -> Self {
        Self {
            screen: Screen::Main,
            active_panel: ActivePanel::Groups,
            client,
            log_groups: StatefulList::new(),
            log_streams: StatefulList::new(),
            log_events: StatefulList::new(),
            selected_event: None,
            filter_input: None,
            filter_editing: false,
            filter_buffer: String::new(),
            viewer_scroll: 0,
            last_selected_group: None,
            needs_clear: false,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // Initial load of log groups
        self.load_log_groups().await?;

        loop {
            if self.needs_clear {
                terminal.clear()?;
                self.needs_clear = false;
            }
            terminal.draw(|f| ui::draw(f, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if self.handle_key(key.code).await? {
                        break;
                    }
                }
            }

            // Check if selected group changed and reload streams
            self.check_group_change().await?;

            // Pagination triggers
            self.check_pagination().await?;
        }

        Ok(())
    }

    async fn handle_key(&mut self, code: KeyCode) -> Result<bool> {
        match self.screen {
            Screen::Main => self.handle_main_key(code).await,
            Screen::Events => self.handle_events_key(code).await,
            Screen::Viewer => self.handle_viewer_key(code).await,
        }
    }

    async fn handle_main_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('l') => {
                self.active_panel = ActivePanel::Streams
            },
            KeyCode::Char('h') => {
                self.active_panel = ActivePanel::Groups
            },
            KeyCode::Char('j') | KeyCode::Down => match self.active_panel {
                ActivePanel::Groups => self.log_groups.next(),
                ActivePanel::Streams => self.log_streams.next(),
            },
            KeyCode::Char('k') | KeyCode::Up => match self.active_panel {
                ActivePanel::Groups => self.log_groups.previous(),
                ActivePanel::Streams => self.log_streams.previous(),
            },
            KeyCode::Enter => {
                if self.active_panel == ActivePanel::Streams
                    && !self.log_streams.items.is_empty()
                    && self.log_streams.state.selected().is_some()
                {
                    self.screen = Screen::Events;
                    self.needs_clear = true;
                    self.log_events = StatefulList::new();
                    self.filter_input = None;
                    self.load_log_events().await?;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_events_key(&mut self, code: KeyCode) -> Result<bool> {
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
        } else {
            match code {
                KeyCode::Char('q') => {
                    self.screen = Screen::Main;
                    self.needs_clear = true;
                }
                KeyCode::Char('j') | KeyCode::Down => self.log_events.next(),
                KeyCode::Char('k') | KeyCode::Up => self.log_events.previous(),
                KeyCode::Char('/') => {
                    self.filter_editing = true;
                    self.filter_buffer = self.filter_input.clone().unwrap_or_default();
                }
                KeyCode::Enter => {
                    if let Some(event) = self.log_events.selected().cloned() {
                        self.selected_event = Some(event);
                        self.viewer_scroll = 0;
                        self.screen = Screen::Viewer;
                        self.needs_clear = true;
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    }

    async fn handle_viewer_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Char('q') => {
                self.screen = Screen::Events;
                self.needs_clear = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.viewer_scroll = self.viewer_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.viewer_scroll = self.viewer_scroll.saturating_sub(1);
            }
            _ => {}
        }
        Ok(false)
    }

    async fn check_group_change(&mut self) -> Result<()> {
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

    async fn check_pagination(&mut self) -> Result<()> {
        if self.screen == Screen::Main {
            // Groups pagination
            if let Some(idx) = self.log_groups.selected_index() {
                let len = self.log_groups.items.len();
                if len > 0 && idx + 5 >= len && self.log_groups.next_token.is_some() && !self.log_groups.loading {
                    self.load_more_groups().await?;
                }
            }
            // Streams pagination
            if self.active_panel == ActivePanel::Streams {
                if let Some(idx) = self.log_streams.selected_index() {
                    let len = self.log_streams.items.len();
                    if len > 0 && idx + 5 >= len && self.log_streams.next_token.is_some() && !self.log_streams.loading {
                        self.load_more_streams().await?;
                    }
                }
            }
        }
        if self.screen == Screen::Events {
            if let Some(idx) = self.log_events.selected_index() {
                let len = self.log_events.items.len();
                if len > 0 && idx + 5 >= len && self.log_events.next_token.is_some() && !self.log_events.loading {
                    self.load_more_events().await?;
                }
            }
        }
        Ok(())
    }

    async fn load_log_groups(&mut self) -> Result<()> {
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
        Ok(())
    }

    async fn load_log_streams(&mut self) -> Result<()> {
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
        Ok(())
    }

    pub async fn load_log_events(&mut self) -> Result<()> {
        let group_name = match self.log_groups.selected() {
            Some(g) => g.name.clone(),
            None => return Ok(()),
        };
        let stream_name = match self.log_streams.selected() {
            Some(s) => s.name.clone(),
            None => return Ok(()),
        };
        let filter = self.filter_input.clone();
        self.log_events.loading = true;
        let (events, token) =
            aws::fetch_log_events(&self.client, &group_name, &stream_name, None, filter, None).await?;
        self.log_events.items = events;
        self.log_events.next_token = token;
        self.log_events.loading = false;
        if !self.log_events.items.is_empty() {
            self.log_events.state.select(Some(0));
        }
        Ok(())
    }

    async fn load_more_events(&mut self) -> Result<()> {
        let group_name = match self.log_groups.selected() {
            Some(g) => g.name.clone(),
            None => return Ok(()),
        };
        let stream_name = match self.log_streams.selected() {
            Some(s) => s.name.clone(),
            None => return Ok(()),
        };
        let filter = self.filter_input.clone();
        let token = self.log_events.next_token.clone();
        self.log_events.loading = true;
        let (events, next) =
            aws::fetch_log_events(&self.client, &group_name, &stream_name, None, filter, token).await?;
        self.log_events.items.extend(events);
        self.log_events.next_token = next;
        self.log_events.loading = false;
        Ok(())
    }
}
