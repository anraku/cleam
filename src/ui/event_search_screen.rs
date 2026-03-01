use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::screen::EventSearchScreen;

pub fn draw(f: &mut Frame, screen: &mut EventSearchScreen) {
    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    // Header
    let header = Paragraph::new(format!(
        " Log Event Search  │  Group: {}",
        screen.group_name
    ))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(header, outer[0]);

    // Body: form fields
    let body_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // start
            Constraint::Length(3), // end
            Constraint::Length(3), // pattern
            Constraint::Length(1), // error
            Constraint::Min(0),    // padding
        ])
        .split(outer[1]);

    render_field(
        f,
        body_layout[0],
        "開始日時 (Start)",
        &screen.event_search_start,
        screen.event_search_focused == 0,
    );
    render_field(
        f,
        body_layout[1],
        "終了日時 (End)",
        &screen.event_search_end,
        screen.event_search_focused == 1,
    );
    render_field(
        f,
        body_layout[2],
        "Filter Pattern",
        &screen.event_search_pattern,
        screen.event_search_focused == 2,
    );

    // Error message
    if let Some(err) = &screen.event_search_error {
        let error_line = Paragraph::new(Line::from(vec![
            Span::styled(
                " ✗ ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(err.as_str(), Style::default().fg(Color::Red)),
        ]));
        f.render_widget(error_line, body_layout[3]);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" 次のフィールド  "),
        Span::styled("[Shift+Tab]", Style::default().fg(Color::Yellow)),
        Span::raw(" 前のフィールド  "),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::raw(" 検索  "),
        Span::styled("[q/Esc]", Style::default().fg(Color::Yellow)),
        Span::raw(" 戻る"),
    ]))
    .style(Style::default().bg(Color::Rgb(30, 30, 30)));
    f.render_widget(footer, outer[2]);
}

fn render_field(f: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let label_color = if focused { Color::Cyan } else { Color::Gray };
    let cursor = if focused {
        Span::styled("█", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("")
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", label),
            Style::default().fg(label_color),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let content = Paragraph::new(Line::from(vec![Span::raw(value.to_owned()), cursor]));
    f.render_widget(content, inner);
}
