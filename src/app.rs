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

pub struct StatefulList<T> {
    pub items: Vec<T>,
    pub state: ratatui::widgets::ListState,
    pub next_token: Option<String>,
    pub loading: bool,
    pub visible_indices: Option<Vec<usize>>,
}

impl<T> StatefulList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ratatui::widgets::ListState::default(),
            next_token: None,
            loading: false,
            visible_indices: None,
        }
    }

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

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected(&self) -> Option<&T> {
        self.state
            .selected()
            .and_then(|i| match &self.visible_indices {
                Some(v) => v.get(i).and_then(|&orig| self.items.get(orig)),
                None => self.items.get(i),
            })
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state
            .selected()
            .and_then(|i| match &self.visible_indices {
                Some(v) => v.get(i).copied(),
                None => Some(i),
            })
    }

    pub fn visible_items(&self) -> Vec<&T> {
        match &self.visible_indices {
            Some(v) => v.iter().filter_map(|&i| self.items.get(i)).collect(),
            None => self.items.iter().collect(),
        }
    }
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
    pub client: Arc<Client>,
    pub screen: CurrentScreen,
    pub needs_clear: bool,
}

impl App {
    pub fn new(client: Client) -> Self {
        let client = Arc::new(client);
        let main_screen = MainScreen::new(Arc::clone(&client));
        Self {
            client,
            screen: CurrentScreen::Main(main_screen),
            needs_clear: false,
        }
    }

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
