use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::screen::GroupEventsScreen;

pub fn draw(f: &mut Frame, screen: &mut GroupEventsScreen) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // list
            Constraint::Length(1), // footer
        ])
        .split(area);

    // Header: group name + search condition summary
    let start_disp = if screen.start_display.is_empty() {
        "*"
    } else {
        screen.start_display.as_str()
    };
    let end_disp = if screen.end_display.is_empty() {
        "*"
    } else {
        screen.end_display.as_str()
    };
    let pattern_disp = if screen.pattern_display.is_empty() {
        String::new()
    } else {
        format!("  │  pattern: {}", screen.pattern_display)
    };
    let header_text = format!(
        " {}  │  {} → {}{}",
        screen.group_name, start_disp, end_disp, pattern_disp
    );
    let header =
        Paragraph::new(header_text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Events list
    let loading = screen.log_events.loading;
    let block_title = if loading {
        " Group Events (loading…) "
    } else {
        " Group Events "
    };
    let block = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if screen.log_events.items.is_empty() && !loading {
        let msg = Paragraph::new("  No events found.").block(block);
        f.render_widget(msg, chunks[1]);
    } else {
        let ts_width: usize = 23;
        let available = (chunks[1].width as usize).saturating_sub(2 + 3 + ts_width + 2);

        let items: Vec<ListItem> = screen
            .log_events
            .items
            .iter()
            .map(|e| {
                let ts = format_timestamp(e.timestamp);
                let joined = e
                    .message
                    .lines()
                    .map(|l| l.trim().replace('\t', " "))
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
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

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [j/k ↑↓]", Style::default().fg(Color::Yellow)),
        Span::raw(" スクロール  "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" 詳細  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" 検索に戻る"),
    ]))
    .style(Style::default().bg(Color::Rgb(30, 30, 30)));
    f.render_widget(footer, chunks[2]);
}

fn format_timestamp(ts_ms: i64) -> String {
    match jiff::Timestamp::from_millisecond(ts_ms) {
        Ok(ts) => {
            let ms = ts_ms.rem_euclid(1000);
            format!("{}.{:03}", ts.strftime("%Y-%m-%d %H:%M:%S"), ms)
        }
        Err(_) => String::from("0000-00-00 00:00:00.000"),
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let mut result: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

fn colorize_level_keyword(line: &str) -> Vec<Span<'static>> {
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
    vec![Span::raw(line.to_owned())]
}
