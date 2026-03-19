use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::{
    app::App,
    tools::{ToolEntry, ToolStatus},
};

pub(crate) fn build_tool_summary_lines(app: &App) -> Vec<Line<'static>> {
    if let Some(entry) = app.tools.latest_running() {
        return vec![
            Line::from(vec![
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
            ]),
            Line::from(Span::styled(
                format!("  {}", entry.detail),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];
    }

    let Some(entry) = app.tools.latest_entry() else {
        return Vec::new();
    };

    let (headline, color) = summary_headline(entry);
    vec![
        Line::from(vec![
            Span::styled(
                headline,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(entry.status.label(), Style::default().fg(color)),
        ]),
        Line::from(Span::styled(
            format!("  {}", entry.detail),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ]
}

pub(crate) fn build_transcript_tool_lines(app: &App) -> Vec<Line<'static>> {
    let Some(entry) = app.tools.latest_running().or_else(|| app.tools.latest_entry()) else {
        return Vec::new();
    };

    let (headline, color) = transcript_headline(entry);
    let mut lines = vec![
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
        ]),
        Line::from(Span::styled(
            format!("  {headline}"),
            Style::default().fg(color),
        )),
    ];

    if !entry.detail.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  {}", entry.detail),
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines
}

fn summary_headline(entry: &ToolEntry) -> (String, Color) {
    match entry.status {
        ToolStatus::Success => (format!("Last tool: {}", entry.name), Color::Green),
        ToolStatus::Failed => (format!("Last tool failed: {}", entry.name), Color::Red),
        ToolStatus::Timeout => (format!("Last tool timed out: {}", entry.name), Color::Magenta),
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
