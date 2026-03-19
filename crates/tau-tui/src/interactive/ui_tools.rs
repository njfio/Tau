use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::app::{App, FocusPanel};
use super::tools::ToolStatus;

pub(crate) fn render_tool_panel(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == FocusPanel::Tools {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(
        " Tools ({} active / {} total) ",
        app.tools.active_count(),
        app.tools.total_count()
    );

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tools.entries().is_empty() {
        let empty =
            Paragraph::new("No tool executions yet.").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    }

    let visible = inner.height as usize;
    let entries = app.tools.entries();
    let start = entries.len().saturating_sub(visible);
    let items: Vec<ListItem> = entries[start..]
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
                Span::styled(&entry.name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    truncate(
                        &entry.detail,
                        (inner.width as usize).saturating_sub(entry.name.len() + 8),
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    if max > 3 {
        return format!("{}...", &value[..max - 3]);
    }
    value[..max].to_string()
}
