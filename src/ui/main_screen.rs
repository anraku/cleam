use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use jiff::Timestamp;

use crate::app::{ActivePanel, App};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Layout: header / panels / [search bar] / footer
    let show_search_bar = app.main_search_active || !app.main_search_query.is_empty();
    let constraints = if show_search_bar {
        vec![
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else {
        vec![
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Header
    let header = Paragraph::new(format!(
        " cleam  |  {}",
        if app.log_groups.loading { "Loading..." } else { "AWS CloudWatch Logs" }
    ))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, chunks[0]);

    // Two-pane layout
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    // --- Groups pane ---
    let groups_active = app.active_panel == ActivePanel::Groups;
    let groups_border_style = if groups_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let groups_block = Block::default()
        .title(" Log Groups ")
        .borders(Borders::ALL)
        .border_style(groups_border_style);

    let group_names: Vec<String> = app.log_groups.visible_items()
        .into_iter()
        .map(|g| g.name.clone())
        .collect();
    let groups_filtered = app.log_groups.visible_indices.is_some();

    let group_items: Vec<ListItem> = group_names.iter()
        .map(|n| ListItem::new(n.as_str()))
        .collect();

    if group_items.is_empty() && groups_filtered {
        let no_match = Paragraph::new("No matches").block(groups_block);
        f.render_widget(no_match, panes[0]);
    } else {
        let groups_list = List::new(group_items)
            .block(groups_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(groups_list, panes[0], &mut app.log_groups.state);
    }

    // --- Streams pane ---
    let streams_active = app.active_panel == ActivePanel::Streams;
    let streams_border_style = if streams_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let streams_title = match app.log_groups.selected() {
        Some(g) => format!(" Streams: {} ", g.name),
        None => " Log Streams ".to_string(),
    };
    let streams_block = Block::default()
        .title(streams_title)
        .borders(Borders::ALL)
        .border_style(streams_border_style);

    let stream_entries: Vec<(String, String)> = app.log_streams.visible_items()
        .into_iter()
        .map(|s| {
            let time_str = match s.last_event_time {
                Some(ms) => {
                    let Ok(ts) = Timestamp::from_millisecond(ms) else {
                        return ("--/-- --:--".to_string(), s.name.clone());
                    };
                    let dt = ts.to_zoned(jiff::tz::TimeZone::UTC).to_string();
                    dt
                }
                None => "--/-- --:--".to_string(),
            };
            (time_str, s.name.clone())
        })
        .collect();
    let streams_filtered = app.log_streams.visible_indices.is_some();

    let stream_items: Vec<ListItem> = stream_entries.iter()
        .map(|(time_str, name)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", time_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(name.as_str()),
            ]))
        })
        .collect();

    if stream_items.is_empty() && streams_filtered {
        let no_match = Paragraph::new("No matches").block(streams_block);
        f.render_widget(no_match, panes[1]);
    } else if app.log_streams.items.is_empty() && !app.log_streams.loading {
        let empty = Paragraph::new(if app.log_groups.items.is_empty() {
            "No log groups found"
        } else {
            "No log streams found"
        })
        .block(streams_block);
        f.render_widget(empty, panes[1]);
    } else {
        let streams_list = List::new(stream_items)
            .block(streams_block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(streams_list, panes[1], &mut app.log_streams.state);
    }

    // --- Search bar (shown when search is active or query is non-empty) ---
    if show_search_bar {
        let search_text = if app.main_search_active {
            format!(" Search: {}_", app.main_search_query)
        } else {
            format!(" Search: {}", app.main_search_query)
        };
        let search_bar = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
        f.render_widget(search_bar, chunks[2]);
    }

    // --- Footer ---
    let footer_idx = if show_search_bar { 3 } else { 2 };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[h]", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch LogGroups  "),
        Span::styled("[l]", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch LogStreams "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" Open Stream  "),
        Span::styled("[/]", Style::default().fg(Color::Yellow)),
        Span::raw(" Search  "),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" Clear Search  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit "),
    ]))
    .style(Style::default().bg(Color::DarkGray));
    f.render_widget(footer, chunks[footer_idx]);
}
