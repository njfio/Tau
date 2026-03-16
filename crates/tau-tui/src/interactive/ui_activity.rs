use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
use super::super::status::AgentStateDisplay;
use super::super::tools::{ToolEntry, ToolStatus};

pub(super) fn attention_height(app: &App) -> u16 {
    if app.approval_request.is_some() || app.status.agent_state == AgentStateDisplay::Error {
        2
    } else {
        0
    }
}

pub(super) fn render_activity_strip(frame: &mut Frame, app: &App, area: Rect) {
    let title = Span::styled(
        " Live activity ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightYellow)
            .add_modifier(Modifier::BOLD),
    );
    let summary = Span::styled(activity_summary(app), Style::default().fg(Color::White));
    let commands = Span::styled(
        " /details  /retry  /interrupt ",
        Style::default().fg(Color::DarkGray),
    );
    let line = Line::from(vec![title, Span::raw(" "), summary, commands]);
    frame.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), area);
}

pub(super) fn render_attention_strip(frame: &mut Frame, app: &App, area: Rect) {
    let line = if let Some(request) = &app.approval_request {
        Line::from(vec![
            Span::styled(
                " Approval required ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(request.summary.clone(), Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled("Approve", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("[Y] approve", Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled("Reject", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled("[N] reject", Style::default().fg(Color::Red)),
        ])
    } else {
        match app.status.agent_state {
            AgentStateDisplay::Error => Line::from(vec![
                Span::styled(
                    " Attention ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("The last turn failed.", Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled("Retry turn", Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled("Open details", Style::default().fg(Color::Cyan)),
            ]),
            _ => Line::default(),
        }
    };
    frame.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), area);
}

fn activity_summary(app: &App) -> String {
    match app.status.agent_state {
        AgentStateDisplay::Idle => "Ready for the next prompt.".to_string(),
        AgentStateDisplay::Thinking => "Thinking through the next step.".to_string(),
        AgentStateDisplay::Streaming => {
            "Streaming assistant output into the transcript.".to_string()
        }
        AgentStateDisplay::Error => "Last turn failed. Open details or retry.".to_string(),
        AgentStateDisplay::ToolExec => latest_running_tool(app)
            .map(|tool| format!("Running tool: {}.", tool.name))
            .unwrap_or_else(|| "Running a tool call.".to_string()),
    }
}

fn latest_running_tool(app: &App) -> Option<&ToolEntry> {
    app.tools
        .entries()
        .iter()
        .rev()
        .find(|entry| entry.status == ToolStatus::Running)
}
