use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{ActivePanel, App};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Header area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    // Header
    let header = ratatui::widgets::Paragraph::new(format!(
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

    // Groups pane
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

    let group_items: Vec<ListItem> = app
        .log_groups
        .items
        .iter()
        .map(|g| ListItem::new(g.name.as_str()))
        .collect();

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

    // Streams pane
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

    if app.log_streams.items.is_empty() && !app.log_streams.loading {
        let empty = ratatui::widgets::Paragraph::new(if app.log_groups.items.is_empty() {
            "No log groups found"
        } else {
            "No log streams found"
        })
        .block(streams_block);
        f.render_widget(empty, panes[1]);
    } else {
        let stream_items: Vec<ListItem> = app
            .log_streams
            .items
            .iter()
            .map(|s| ListItem::new(s.name.as_str()))
            .collect();

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

    // Footer
    let footer = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled(" [Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" Switch Panel  "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" Open Stream  "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit "),
    ]))
    .style(Style::default().bg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}
