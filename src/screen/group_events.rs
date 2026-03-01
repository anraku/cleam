use anyhow::Result;
use aws_sdk_cloudwatchlogs::Client;
use crossterm::event::KeyCode;
use std::sync::Arc;

use super::{CurrentScreen, NavigateTo, ScreenAction};
use crate::app::{LogEvent, StatefulList};
use crate::aws;

pub struct GroupEventsScreen {
    pub client: Arc<Client>,
    pub log_events: StatefulList<LogEvent>,
    pub group_name: String,
    pub start_display: String,
    pub end_display: String,
    pub pattern_display: String,
    pub origin: Option<Box<CurrentScreen>>,
}

impl GroupEventsScreen {
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
