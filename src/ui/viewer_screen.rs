use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // content
            Constraint::Length(1), // footer
        ])
        .split(area);

    // Header
    let ts_display = app
        .selected_event
        .as_ref()
        .map(|e| format_timestamp(e.timestamp))
        .unwrap_or_default();
    let header = ratatui::widgets::Paragraph::new(format!(" Event: {ts_display}"))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Content
    let message = app
        .selected_event
        .as_ref()
        .map(|e| e.message.as_str())
        .unwrap_or("");

    let block = Block::default()
        .title(" Log Event (read-only) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let content = Paragraph::new(message)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.viewer_scroll, 0));

    f.render_widget(content, chunks[1]);

    // Footer
    let footer = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled(" [j/k]", Style::default().fg(Color::Yellow)),
        Span::raw(" Scroll  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Back "),
    ]))
    .style(Style::default().bg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

fn format_timestamp(ts_ms: i64) -> String {
    let secs = ts_ms / 1000;
    let dt = DateTime::<Utc>::from_timestamp(secs, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
