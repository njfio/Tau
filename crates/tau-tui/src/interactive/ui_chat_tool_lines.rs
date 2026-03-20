//! Tool-related chat lines shared by the transcript and summary strip.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::{
    app::App,
    build_status::current_build_status,
    tools::{ToolEntry, ToolStatus},
};

pub(crate) fn build_tool_summary_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = build_build_status_lines(app);
    lines.extend(build_mutating_progress_lines(app));
    if let Some(entry) = app.tools.latest_running() {
        lines.extend(running_summary_lines(entry));
        return lines;
    }

    let Some(entry) = app.tools.latest_entry() else {
        return lines;
    };

    lines.extend(terminal_summary_lines(entry));
    lines
}

pub(crate) fn build_transcript_tool_lines(app: &App) -> Vec<Line<'static>> {
    let Some(entry) = app
        .tools
        .latest_running()
        .or_else(|| app.tools.latest_entry())
    else {
        return Vec::new();
    };

    let (headline, color) = transcript_headline(entry);
    let mut lines = vec![
        tool_header_line(entry),
        Line::from(Span::styled(
            format!("  {headline}"),
            Style::default().fg(color),
        )),
    ];
    push_tool_detail_line(&mut lines, &entry.detail);
    lines.push(Line::from(""));
    lines
}

fn running_summary_lines(entry: &ToolEntry) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("Running tool: {}", entry.name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            entry.status.accent_name().to_uppercase(),
            Style::default().fg(Color::Yellow),
        ),
    ])];
    push_tool_detail_line(&mut lines, &entry.detail);
    lines.push(Line::from(""));
    lines
}

fn build_build_status_lines(app: &App) -> Vec<Line<'static>> {
    let Some(status) = current_build_status(
        app.status.agent_state,
        app.latest_user_prompt(),
        app.current_turn_tools(),
    ) else {
        return Vec::new();
    };

    vec![
        Line::from(vec![
            Span::styled("Build status: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                status.label(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ]
}

fn build_mutating_progress_lines(app: &App) -> Vec<Line<'static>> {
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

fn terminal_summary_lines(entry: &ToolEntry) -> Vec<Line<'static>> {
    let (headline, color) = summary_headline(entry);
    let mut lines = vec![Line::from(vec![
        Span::styled(
            headline,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(entry.status.label(), Style::default().fg(color)),
    ])];
    push_tool_detail_line(&mut lines, &entry.detail);
    lines.push(Line::from(""));
    lines
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

fn summary_headline(entry: &ToolEntry) -> (String, Color) {
    match entry.status {
        ToolStatus::Success => (format!("Last tool: {}", entry.name), Color::Green),
        ToolStatus::Failed => (format!("Last tool failed: {}", entry.name), Color::Red),
        ToolStatus::Timeout => (
            format!("Last tool timed out: {}", entry.name),
            Color::Magenta,
        ),
        ToolStatus::Running => unreachable!("running tool handled earlier"),
    }
}

fn transcript_headline(entry: &ToolEntry) -> (String, Color) {
    match entry.status {
        ToolStatus::Running => (format!("{} running", entry.name), Color::Yellow),
        ToolStatus::Success => (format!("{} ok", entry.name), Color::Green),
        ToolStatus::Failed => (format!("{} failed", entry.name), Color::Red),
        ToolStatus::Timeout => (format!("{} timed out", entry.name), Color::Magenta),
    }
}

fn tool_header_line(entry: &ToolEntry) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("[{}] ", entry.timestamp),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            "Tool: ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn push_tool_detail_line(lines: &mut Vec<Line<'static>>, detail: &str) {
    if detail.is_empty() {
        return;
    }

    lines.push(Line::from(Span::styled(
        format!("  {detail}"),
        Style::default().fg(Color::DarkGray),
    )));
}
