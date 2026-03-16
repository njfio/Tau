use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use super::super::app::{App, FocusPanel};
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
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::LEFT)
        .border_style(border_style)
        .style(Style::default().bg(Color::Rgb(12, 14, 18)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(List::new(detail_items(app, inner.width as usize)), inner);
}

fn detail_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    let mut items = vec![
        section_item("Tool activity"),
        detail_line_item(format!(
            "{} active / {} total",
            app.tools.active_count(),
            app.tools.total_count()
        )),
    ];
    items.extend(recent_tool_items(app, max_width));
    items.extend([
        section_item("Memory"),
        detail_line_item("No stored memory yet.".to_string()),
        section_item("Cortex"),
        detail_line_item("Observer idle until a turn completes.".to_string()),
        section_item("Sessions"),
        detail_line_item(format!("Current session: {}", app.config.session_key)),
    ]);
    items
}

fn recent_tool_items(app: &App, max_width: usize) -> Vec<ListItem<'static>> {
    let entries = app.tools.entries();
    if entries.is_empty() {
        return vec![detail_line_item("No tool executions yet.".to_string())];
    }
    let start = entries.len().saturating_sub(3);
    entries[start..]
        .iter()
        .map(|entry| {
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
                    truncate(&entry.detail, max_width.saturating_sub(entry.name.len() + 8)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect()
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
