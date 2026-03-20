use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
use super::super::status::AgentStateDisplay;
use super::build_evidence::build_evidence_state;
use super::shared::{action, badge, latest_running_tool};

pub(super) fn attention_height(app: &App) -> u16 {
    if app.approval_request.is_some() || app.status.agent_state == AgentStateDisplay::Error {
        2
    } else {
        0
    }
}

pub(super) fn render_activity_strip(frame: &mut Frame, app: &App, area: Rect) {
    let line = Line::from(activity_spans(app));
    frame.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), area);
}

pub(super) fn render_attention_strip(frame: &mut Frame, app: &App, area: Rect) {
    let line = if let Some(request) = &app.approval_request {
        approval_attention_line(request.summary.as_str())
    } else {
        match app.status.agent_state {
            AgentStateDisplay::Error => error_attention_line(),
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

fn activity_spans(app: &App) -> Vec<Span<'static>> {
    let mut spans = vec![
        badge(" Live activity ", Color::LightYellow),
        Span::raw(" "),
        state_chip(app.status.agent_state),
        Span::raw(" "),
        Span::styled(activity_summary(app), Style::default().fg(Color::White)),
    ];
    spans.extend(operator_activity_spans(app));
    if app.status.agent_state == AgentStateDisplay::ToolExec {
        spans.extend(tool_activity_spans(app));
    }
    spans.extend(build_evidence_spans(app));
    spans.extend([
        Span::raw(" "),
        action("/details", Color::DarkGray),
        Span::raw("  "),
        action("/retry", Color::DarkGray),
        Span::raw("  "),
        action("/interrupt", Color::DarkGray),
    ]);
    spans
}

fn approval_attention_line(summary: &str) -> Line<'static> {
    Line::from(vec![
        badge(" Approval required ", Color::Yellow),
        Span::raw(" "),
        Span::styled(summary.to_string(), Style::default().fg(Color::White)),
        Span::raw(" "),
        action("Approve", Color::Green),
        Span::raw(" "),
        action("[Y] approve", Color::Green),
        Span::raw("  "),
        action("Reject", Color::Red),
        Span::raw(" "),
        action("[N] reject", Color::Red),
    ])
}

fn error_attention_line() -> Line<'static> {
    Line::from(vec![
        badge(" Attention ", Color::LightRed),
        Span::raw(" "),
        Span::styled("The last turn failed.", Style::default().fg(Color::White)),
        Span::raw(" "),
        action("Retry turn", Color::Yellow),
        Span::raw(" "),
        action("[r] retry", Color::Yellow),
        Span::raw("  "),
        action("Open details", Color::Cyan),
        Span::raw(" "),
        action("[/details]", Color::Cyan),
    ])
}

fn operator_activity_spans(app: &App) -> Vec<Span<'static>> {
    let Some(state) = &app.last_operator_state else {
        return Vec::new();
    };
    let phase = state.phase.as_deref().unwrap_or(state.status.as_str());
    let mut spans = vec![
        Span::raw(" "),
        action(&format!("{}:{phase}", state.entity), Color::DarkGray),
    ];
    if let Some(kind) = &state.artifact_kind {
        spans.push(Span::raw(" "));
        spans.push(action(kind, Color::Green));
    }
    if let Some(code) = &state.reason_code {
        spans.push(Span::raw(" "));
        spans.push(action(code, Color::LightRed));
    }
    spans
}

fn tool_activity_spans(app: &App) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let count = app.tools.active_count();
    if count > 0 {
        spans.push(Span::raw(" "));
        spans.push(action(&format!("{count} active"), Color::Cyan));
    }
    if let Some(tool) = latest_running_tool(app) {
        spans.push(Span::raw(" "));
        spans.push(action(&tool.name, Color::Yellow));
    }
    spans
}

fn build_evidence_spans(app: &App) -> Vec<Span<'static>> {
    let Some(state) = build_evidence_state(app) else {
        return Vec::new();
    };
    vec![
        Span::raw(" "),
        action(state.activity_text(), evidence_color(state)),
    ]
}

fn evidence_color(state: super::build_evidence::BuildEvidenceState) -> Color {
    match state {
        super::build_evidence::BuildEvidenceState::NoMutatingEvidenceYet => Color::LightRed,
        super::build_evidence::BuildEvidenceState::ReadOnlySoFar => Color::Yellow,
        super::build_evidence::BuildEvidenceState::MutatingEvidenceConfirmed => Color::Green,
    }
}

fn state_chip(state: AgentStateDisplay) -> Span<'static> {
    let (label, color) = match state {
        AgentStateDisplay::Idle => ("idle", Color::DarkGray),
        AgentStateDisplay::Thinking => ("thinking", Color::Yellow),
        AgentStateDisplay::ToolExec => ("tool", Color::Cyan),
        AgentStateDisplay::Streaming => ("stream", Color::Green),
        AgentStateDisplay::Error => ("error", Color::LightRed),
    };
    badge(&format!(" {label} "), color)
}
