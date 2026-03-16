use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::super::app::{App, DetailSection, FocusPanel};
use super::super::tools::ToolStatus;

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
        DetailSection::Memory => memory_items(),
        DetailSection::Cortex => simple_items("Cortex", "Observer idle until a turn completes."),
        DetailSection::Sessions => session_items(app),
    }
}

fn tool_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    let mut items = vec![
        section_item("Tool activity"),
        detail_line_item(format!(
            "{} active / {} total",
            app.tools.active_count(),
            app.tools.total_count()
        )),
        detail_line_item("Open Memory, Cortex, or Sessions from the tabs above.".to_string()),
    ];
    let entries = app.tools.entries();
    if entries.is_empty() {
        items.push(detail_line_item("No tool executions yet.".to_string()));
        return items;
    }
    let start = entries.len().saturating_sub(4);
    items.extend(entries[start..].iter().map(|entry| {
        let status_style = match entry.status {
            ToolStatus::Running => Style::default().fg(Color::Yellow),
            ToolStatus::Success => Style::default().fg(Color::Green),
            ToolStatus::Failed => Style::default().fg(Color::Red),
            ToolStatus::Timeout => Style::default().fg(Color::Magenta),
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("[{}] ", entry.status.label()), status_style),
            Span::styled(entry.name.clone(), Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(
                truncate(
                    &entry.detail,
                    max_width.saturating_sub(entry.name.len() + 8),
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
    }));
    items
}

fn simple_items(title: &str, message: &str) -> Vec<ListItem<'static>> {
    vec![section_item(title), detail_line_item(message.to_string())]
}

fn memory_items() -> Vec<ListItem<'static>> {
    vec![
        section_item("Memory"),
        detail_line_item("Status: degraded".to_string()),
        detail_line_item("shared state unavailable".to_string()),
        detail_line_item("No stored memory yet.".to_string()),
    ]
}

fn session_items(app: &App) -> Vec<ListItem<'static>> {
    vec![
        section_item("Sessions"),
        detail_line_item(format!("Current session: {}", app.config.session_key)),
        detail_line_item(format!("Messages: {}", app.chat.len())),
        detail_line_item(format!("Tokens: {}", app.status.total_tokens)),
        detail_line_item(format!(
            "Approvals pending: {}",
            usize::from(app.approval_request.is_some())
        )),
    ]
}

fn section_item(title: &str) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )))
}

fn detail_line_item(text: String) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(Color::Gray),
    )))
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        return input.to_string();
    }
    if max > 3 {
        return format!("{}...", &input[..max - 3]);
    }
    input[..max].to_string()
}
