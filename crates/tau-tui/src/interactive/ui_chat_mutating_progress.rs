use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::{
    app::App,
    tools::{ToolEntry, ToolStatus},
};

pub(crate) fn build_mutating_progress_lines(app: &App) -> Vec<Line<'static>> {
    let current_turn_tools = app.current_turn_tools();
    if let Some(entry) = latest_running_mutating_entry(current_turn_tools) {
        return running_mutating_lines(entry);
    }

    let Some(entry) = latest_successful_mutating_entry(current_turn_tools) else {
        return Vec::new();
    };
    if entry.detail.is_empty() {
        return Vec::new();
    }

    successful_mutating_lines(entry)
}

fn latest_running_mutating_entry(entries: &[ToolEntry]) -> Option<&ToolEntry> {
    entries
        .iter()
        .rev()
        .find(|entry| entry.status == ToolStatus::Running && is_mutating_tool(entry))
}

fn latest_successful_mutating_entry(entries: &[ToolEntry]) -> Option<&ToolEntry> {
    entries
        .iter()
        .rev()
        .find(|entry| entry.status == ToolStatus::Success && is_mutating_tool(entry))
}

fn is_mutating_tool(entry: &ToolEntry) -> bool {
    matches!(entry.name.as_str(), "write" | "edit")
}

fn running_mutating_lines(entry: &ToolEntry) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Mutating now: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                entry.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!("  {}", entry.detail),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ]
}

fn successful_mutating_lines(entry: &ToolEntry) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled(
                format!("Latest {} target: ", entry.name),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                entry.detail.clone(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ]
}
