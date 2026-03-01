//! イベント検索フォームスクリーンの状態管理。
//!
//! 開始日時・終了日時・フィルタパターンを入力してグループ横断検索を実行します。

use anyhow::Result;
use crossterm::event::KeyCode;

use super::{CurrentScreen, NavigateTo, ScreenAction};

/// 時間範囲とフィルタパターンでイベントを検索するフォームスクリーン。
///
/// `Tab`/`BackTab` でフィールド間を移動し、`Enter` で検索を実行します。
/// 日時は `YYYY-MM-DD HH:MM:SS` 形式（UTC）で入力します。
pub struct EventSearchScreen {
    /// 検索対象のロググループ名
    pub group_name: String,
    /// 検索開始日時の入力文字列（`YYYY-MM-DD HH:MM:SS`）
    pub event_search_start: String,
    /// 検索終了日時の入力文字列（`YYYY-MM-DD HH:MM:SS`）
    pub event_search_end: String,
    /// CloudWatch Logs フィルタパターンの入力文字列
    pub event_search_pattern: String,
    /// 現在フォーカスされているフィールドのインデックス（0: 開始、1: 終了、2: パターン）
    pub event_search_focused: u8,
    /// バリデーションエラーメッセージ
    pub event_search_error: Option<String>,
    /// 前の画面（`q`/`Esc` で戻るため保持）
    pub origin: Option<Box<CurrentScreen>>,
}

impl EventSearchScreen {
    /// 新しい [`EventSearchScreen`] を生成します。
    ///
    /// 開始日時は現在時刻の 1 時間前、終了日時は現在時刻で初期化されます。
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

    /// キー入力を処理して [`ScreenAction`] を返します。
    ///
    /// `Enter` でフォームを検証し、成功すれば [`NavigateTo::NewGroupEvents`] を返します。
    /// 日時の解析に失敗した場合は `event_search_error` にエラーメッセージを設定します。
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

/// `YYYY-MM-DD HH:MM:SS` 形式の文字列を UTC Unix ミリ秒に変換します。
///
/// # Errors
///
/// - 日時の形式が不正な場合
/// - タイムゾーン変換に失敗した場合
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
