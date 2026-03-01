use anyhow::Result;
use crossterm::event::KeyCode;

use super::{CurrentScreen, NavigateTo, ScreenAction};
use crate::app::LogEvent;

pub struct ViewerScreen {
    pub selected_event: LogEvent,
    pub viewer_scroll: u16,
    pub origin: Option<Box<CurrentScreen>>,
}

impl ViewerScreen {
    pub fn new(event: LogEvent, origin: Box<CurrentScreen>) -> Self {
        Self {
            selected_event: event,
            viewer_scroll: 0,
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
