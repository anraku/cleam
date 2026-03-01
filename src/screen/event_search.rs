use anyhow::Result;
use crossterm::event::KeyCode;

use super::{CurrentScreen, NavigateTo, ScreenAction};

pub struct EventSearchScreen {
    pub group_name: String,
    pub event_search_start: String,
    pub event_search_end: String,
    pub event_search_pattern: String,
    pub event_search_focused: u8,
    pub event_search_error: Option<String>,
    pub origin: Option<Box<CurrentScreen>>,
}

impl EventSearchScreen {
    pub fn new(group_name: String, origin: Box<CurrentScreen>) -> Self {
        let now = jiff::Zoned::now();
        let one_hour_ago = now.saturating_sub(jiff::Span::new().hours(1));
        Self {
            group_name,
            event_search_start: one_hour_ago.strftime("%Y-%m-%d %H:%M:%S").to_string(),
            event_search_end: now.strftime("%Y-%m-%d %H:%M:%S").to_string(),
            event_search_pattern: String::new(),
            event_search_focused: 0,
            event_search_error: None,
            origin: Some(origin),
        }
    }

    pub async fn handle_key(&mut self, code: KeyCode) -> Result<ScreenAction> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if let Some(origin) = self.origin.take() {
                    return Ok(ScreenAction::Navigate(NavigateTo::Restore(origin)));
                }
            }
            KeyCode::Tab => {
                self.event_search_focused = (self.event_search_focused + 1) % 3;
            }
            KeyCode::BackTab => {
                self.event_search_focused = (self.event_search_focused + 2) % 3;
            }
            KeyCode::Backspace => match self.event_search_focused {
                0 => {
                    self.event_search_start.pop();
                }
                1 => {
                    self.event_search_end.pop();
                }
                _ => {
                    self.event_search_pattern.pop();
                }
            },
            KeyCode::Char(c) => match self.event_search_focused {
                0 => self.event_search_start.push(c),
                1 => self.event_search_end.push(c),
                _ => self.event_search_pattern.push(c),
            },
            KeyCode::Enter => {
                let start_ms_result = if self.event_search_start.is_empty() {
                    Ok(None)
                } else {
                    parse_datetime_to_ms(&self.event_search_start).map(Some)
                };
                let end_ms_result = if self.event_search_end.is_empty() {
                    Ok(None)
                } else {
                    parse_datetime_to_ms(&self.event_search_end).map(Some)
                };
                match (start_ms_result, end_ms_result) {
                    (Ok(start_ms), Ok(end_ms)) => {
                        self.event_search_error = None;
                        let start_display = self.event_search_start.clone();
                        let end_display = self.event_search_end.clone();
                        let pattern_display = self.event_search_pattern.clone();
                        let pattern = if self.event_search_pattern.is_empty() {
                            None
                        } else {
                            Some(self.event_search_pattern.clone())
                        };
                        return Ok(ScreenAction::Navigate(NavigateTo::NewGroupEvents {
                            group_name: self.group_name.clone(),
                            start_ms,
                            end_ms,
                            pattern,
                            start_display,
                            end_display,
                            pattern_display,
                        }));
                    }
                    (Err(_), _) => {
                        self.event_search_error =
                            Some("開始日時の形式が不正です（例: 2024-01-01 12:00:00）".to_string());
                    }
                    (_, Err(_)) => {
                        self.event_search_error =
                            Some("終了日時の形式が不正です（例: 2024-01-01 12:00:00）".to_string());
                    }
                }
            }
            _ => {}
        }
        Ok(ScreenAction::None)
    }
}

fn parse_datetime_to_ms(s: &str) -> anyhow::Result<i64> {
    let iso_str = s.replacen(' ', "T", 1);
    let dt: jiff::civil::DateTime = iso_str
        .parse()
        .map_err(|_| anyhow::anyhow!("日時のフォーマットが不正です（例: 2024-01-01 12:00:00）"))?;
    let tz = jiff::tz::TimeZone::UTC;
    let zoned = dt
        .to_zoned(tz)
        .map_err(|e| anyhow::anyhow!("タイムゾーン変換に失敗しました: {}", e))?;
    Ok(zoned.timestamp().as_millisecond())
}
