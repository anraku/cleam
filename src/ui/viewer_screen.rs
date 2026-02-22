use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let ts_display = app
        .selected_event
        .as_ref()
        .map(|e| format_timestamp(e.timestamp))
        .unwrap_or_default();
    let header = Paragraph::new(format!(" {} ", ts_display))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Content
    let message = app
        .selected_event
        .as_ref()
        .map(|e| e.message.as_str())
        .unwrap_or("");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let content_lines = render_message(message);
    let content = Paragraph::new(Text::from(content_lines))
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.viewer_scroll, 0));

    f.render_widget(content, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [j/k ↑↓]", Style::default().fg(Color::Yellow)),
        Span::raw(" scroll  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
    ]))
    .style(Style::default().bg(Color::Rgb(30, 30, 30)));
    f.render_widget(footer, chunks[2]);
}

// ── message rendering ─────────────────────────────────────────────────────────

fn render_message(msg: &str) -> Vec<Line<'static>> {
    let trimmed = msg.trim();

    // Try JSON
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let pretty = serde_json::to_string_pretty(&value).unwrap_or_default();
            return pretty.lines().map(|l| highlight_json_line(l)).collect();
        }
    }

    // Plain text: color based on log level keywords
    msg.lines().map(|l| plain_line(l)).collect()
}

fn plain_line(line: &str) -> Line<'static> {
    let upper = line.to_uppercase();
    let color = if upper.contains("ERROR") || upper.contains("FATAL") || upper.contains("CRITICAL") {
        Color::Red
    } else if upper.contains("WARN") {
        Color::Yellow
    } else if upper.contains("DEBUG") {
        Color::Cyan
    } else if upper.contains("TRACE") {
        Color::Magenta
    } else {
        Color::Reset
    };
    Line::from(Span::styled(line.to_owned(), Style::default().fg(color)))
}

// ── JSON syntax highlight ─────────────────────────────────────────────────────

fn highlight_json_line(line: &str) -> Line<'static> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = " ".repeat(indent_len);
    let content = &line[indent_len..];

    // Strip trailing comma for value analysis
    let (bare, comma) = if content.ends_with(',') {
        (&content[..content.len() - 1], ",")
    } else {
        (content, "")
    };

    let mut spans = vec![Span::raw(indent)];

    if bare == "{" || bare == "}" || bare == "[" || bare == "]" {
        spans.push(Span::styled(
            format!("{}{}", bare, comma),
            Style::default().fg(Color::DarkGray),
        ));
    } else if let Some((key, value)) = split_key_value(bare) {
        // "key": value
        spans.push(Span::styled(
            format!("\"{}\"", key),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(": ", Style::default().fg(Color::DarkGray)));
        spans.extend(value_spans(value, comma));
    } else {
        // Array element or bare value
        spans.extend(value_spans(bare, comma));
    }

    Line::from(spans)
}

/// Returns `Some((key_without_quotes, value_str))` for a `"key": value` line fragment.
fn split_key_value(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with('"') {
        return None;
    }
    let bytes = s.as_bytes();
    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            // i is the closing quote of the key
            let after = s[i + 1..].trim_start();
            if let Some(rest) = after.strip_prefix(':') {
                let value = rest.trim_start();
                let key = &s[1..i]; // between the two quotes
                return Some((key, value));
            }
            return None;
        }
        i += 1;
    }
    None
}

fn value_spans(value: &str, comma: &str) -> Vec<Span<'static>> {
    let (color, text) = if value.starts_with('"') && value.ends_with('"') {
        (Color::Green, value.to_owned())
    } else if value == "true" || value == "false" || value == "null" {
        (Color::Magenta, value.to_owned())
    } else if value.starts_with('{') || value.starts_with('[') {
        (Color::DarkGray, value.to_owned())
    } else if value.parse::<f64>().is_ok() {
        (Color::Yellow, value.to_owned())
    } else {
        (Color::White, value.to_owned())
    };

    let mut spans = vec![Span::styled(text, Style::default().fg(color))];
    if !comma.is_empty() {
        spans.push(Span::styled(",".to_owned(), Style::default().fg(Color::DarkGray)));
    }
    spans
}

fn format_timestamp(ts_ms: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(ts_ms / 1000, ((ts_ms % 1000) * 1_000_000) as u32)
        .unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()
}
