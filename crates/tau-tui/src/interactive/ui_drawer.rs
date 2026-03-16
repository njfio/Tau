use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::super::app::{App, DetailSection, FocusPanel};
use super::drawer_sections::{cortex_items, memory_items, session_items, tool_items};

pub(super) fn render_detail_drawer(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == FocusPanel::Tools {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(Span::styled(
            " Details ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::LEFT)
        .border_style(border_style)
        .style(Style::default().bg(Color::Rgb(12, 14, 18)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render_detail_contents(frame, app, inner);
}

pub(super) fn render_detail_contents(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);
    frame.render_widget(Paragraph::new(tab_line(app)), chunks[0]);
    frame.render_widget(
        List::new(detail_items(app, chunks[1].width as usize)),
        chunks[1],
    );
}

fn tab_line(app: &App) -> Line<'static> {
    let tabs = [
        DetailSection::Tools,
        DetailSection::Memory,
        DetailSection::Cortex,
        DetailSection::Sessions,
    ];
    let spans = tabs
        .into_iter()
        .flat_map(|section| {
            let style = if section == app.detail_section {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            vec![
                Span::styled(format!("[{}]", section.label()), style),
                Span::raw(" "),
            ]
        })
        .collect::<Vec<_>>();
    Line::from(spans)
}

fn detail_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    match app.detail_section {
        DetailSection::Tools => tool_items(app, max_width),
        DetailSection::Memory => memory_items(app),
        DetailSection::Cortex => cortex_items(app),
        DetailSection::Sessions => session_items(app),
    }
}
