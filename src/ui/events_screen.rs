use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
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
    let group_name = app.log_groups.selected().map(|g| g.name.as_str()).unwrap_or("-");
    let stream_name = app.log_streams.selected().map(|s| s.name.as_str()).unwrap_or("-");
    let filter_display = match &app.filter_input {
        Some(f) => format!("  |  filter: {}", f),
        None => String::new(),
    };
    let header_text = format!(" {group_name} > {stream_name}{filter_display}");
    let header = Paragraph::new(header_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Events list
    let loading = app.log_events.loading;
    let block_title = if loading {
        " Log Events (loading...) "
    } else {
        " Log Events "
    };
    let block = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if app.log_events.items.is_empty() && !loading {
        let msg = Paragraph::new("No events in the last hour")
            .block(block);
        f.render_widget(msg, chunks[1]);
    } else {
        let max_width = chunks[1].width.saturating_sub(6) as usize; // borders + highlight symbol
        let items: Vec<ListItem> = app
            .log_events
            .items
            .iter()
            .map(|e| {
                let ts = format_timestamp(e.timestamp);
                let msg = truncate(&e.message.replace('\n', " "), max_width.saturating_sub(ts.len() + 2));
                ListItem::new(format!("{}  {}", ts, msg))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[1], &mut app.log_events.state);
    }

    // Footer / filter input
    if app.filter_editing {
        let input_line = Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(Color::Yellow)),
            Span::raw(app.filter_buffer.as_str()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("[Enter]", Style::default().fg(Color::DarkGray)),
            Span::raw(" confirm  "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" cancel"),
        ]);
        let footer = Paragraph::new(input_line).style(Style::default().bg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    } else {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" [/]", Style::default().fg(Color::Yellow)),
            Span::raw(" Filter  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Open  "),
            Span::styled("[j/k]", Style::default().fg(Color::Yellow)),
            Span::raw(" Scroll  "),
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back "),
        ]))
        .style(Style::default().bg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    }
}

fn format_timestamp(ts_ms: i64) -> String {
    let secs = ts_ms / 1000;
    let dt = DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or_default();
    dt.format("%H:%M:%S").to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max > 3 {
        format!("{}...", &s[..max - 3])
    } else {
        s[..max].to_string()
    }
}
