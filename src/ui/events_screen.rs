use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::screen::EventsScreen;

pub fn draw(f: &mut Frame, screen: &mut EventsScreen) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // list
            Constraint::Length(1), // footer / filter input
        ])
        .split(area);

    // Header
    let filter_display = match &screen.filter_input {
        Some(f) => format!("  │  filter: {}", f),
        None => String::new(),
    };
    let header_text = format!(
        " {} › {}{}",
        screen.group_name, screen.stream_name, filter_display
    );
    let header =
        Paragraph::new(header_text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Events list
    let loading = screen.log_events.loading;
    let block_title = if loading {
        " Events (loading…) "
    } else {
        " Events "
    };
    let block = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if screen.log_events.items.is_empty() && !loading {
        let msg = Paragraph::new("  No events found.").block(block);
        f.render_widget(msg, chunks[1]);
    } else {
        // timestamp col width: "YYYY-MM-DD HH:MM:SS.mmm" = 23
        let ts_width: usize = 23;
        // subtract: borders(2) + highlight symbol "▶ " (▶ renders as 2 cols + space = 3) + separator "  "(2)
        let available = (chunks[1].width as usize).saturating_sub(2 + 3 + ts_width + 2);

        let items: Vec<ListItem> = screen
            .log_events
            .items
            .iter()
            .map(|e| {
                let ts = format_timestamp(e.timestamp);
                // 全行を trim して空行を除き、スペース区切りで1行に結合
                let joined = e
                    .message
                    .lines()
                    .map(|l| l.trim().replace('\t', " "))
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                // 文字数ではなく「表示列数」でtruncate
                let msg = truncate_chars(&joined, available);
                let mut spans = vec![
                    Span::styled(ts, Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                ];
                spans.extend(colorize_level_keyword(&msg));
                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(50, 50, 70))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut screen.log_events.state);
    }

    // Footer / filter input / download input
    if screen.download_editing {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                " save to: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(screen.download_path_buffer.as_str()),
            Span::styled("█", Style::default().fg(Color::Cyan)),
            Span::raw("   "),
            Span::styled("[Enter]", Style::default().fg(Color::DarkGray)),
            Span::raw(" save  "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" cancel"),
        ]))
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));
        f.render_widget(footer, chunks[2]);
    } else if screen.filter_editing {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                " filter: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(screen.filter_buffer.as_str()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::raw("   "),
            Span::styled("[Enter]", Style::default().fg(Color::DarkGray)),
            Span::raw(" apply  "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" cancel"),
        ]))
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));
        f.render_widget(footer, chunks[2]);
    } else if let Some(status) = &screen.download_status {
        let (fg, prefix) = if status.starts_with("Error") {
            (Color::Red, "")
        } else {
            (Color::Green, "")
        };
        let footer = Paragraph::new(Line::from(vec![Span::styled(
            format!(" {}{}", prefix, status),
            Style::default().fg(fg).add_modifier(Modifier::BOLD),
        )]))
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));
        f.render_widget(footer, chunks[2]);
    } else {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" [/]", Style::default().fg(Color::Yellow)),
            Span::raw(" filter  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" open  "),
            Span::styled("[j/k ↑↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" scroll  "),
            Span::styled("[d]", Style::default().fg(Color::Yellow)),
            Span::raw(" download  "),
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ]))
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));
        f.render_widget(footer, chunks[2]);
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn format_timestamp(ts_ms: i64) -> String {
    match jiff::Timestamp::from_millisecond(ts_ms) {
        Ok(ts) => {
            let ms = ts_ms.rem_euclid(1000);
            format!("{}.{:03}", ts.strftime("%Y-%m-%d %H:%M:%S"), ms)
        }
        Err(_) => String::from("0000-00-00 00:00:00.000"),
    }
}

/// Truncate to at most `max` Unicode scalar values, appending `…` if cut.
fn truncate_chars(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let mut result: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

/// Scan `line` for the first log-level keyword and return Spans with only
/// that keyword colored; the rest of the text is left in the default color.
fn colorize_level_keyword(line: &str) -> Vec<Span<'static>> {
    // Keywords ordered so longer matches win (CRITICAL before ERROR, etc.)
    const KEYWORDS: &[(&str, Color)] = &[
        ("CRITICAL", Color::Red),
        ("FATAL", Color::Red),
        ("ERROR", Color::Red),
        ("ERR", Color::Red),
        ("WARNING", Color::Yellow),
        ("WARN", Color::Yellow),
        ("INFO", Color::Green),
        ("DEBUG", Color::Cyan),
        ("TRACE", Color::Magenta),
    ];

    let upper = line.to_uppercase();
    for (kw, color) in KEYWORDS {
        if let Some(pos) = upper.find(kw) {
            let end = pos + kw.len();
            // Use original-case slice for display
            let before = line[..pos].to_owned();
            let keyword = line[pos..end].to_owned();
            let after = line[end..].to_owned();

            let mut spans = Vec::new();
            if !before.is_empty() {
                spans.push(Span::raw(before));
            }
            spans.push(Span::styled(
                keyword,
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ));
            if !after.is_empty() {
                spans.push(Span::raw(after));
            }
            return spans;
        }
    }

    // No keyword found — plain white
    vec![Span::raw(line.to_owned())]
}
