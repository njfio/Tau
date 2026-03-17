use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::super::app::App;
use super::super::status::AgentStateDisplay;
use super::shared::{badge, latest_running_tool};

pub(super) fn run_state_height(app: &App) -> u16 {
    run_state_card(app)
        .map(|card| 5 + u16::from(card.meta.is_some()) + u16::from(card.preview.is_some()))
        .unwrap_or(0)
}

pub(super) fn render_run_state_card(frame: &mut Frame, app: &App, area: Rect) {
    let Some(card) = run_state_card(app) else {
        return;
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(12, 15, 20)));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines = vec![
        Line::from(vec![
            badge(card.title, card.color),
            Span::raw(" "),
            Span::styled(card.context, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(Span::styled(
            card.primary,
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            card.secondary,
            Style::default().fg(Color::Gray),
        )),
    ];
    let lines = extend_with_meta(lines, card.meta);
    let lines = extend_with_preview(lines, card.preview);
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

struct RunStateCard<'a> {
    title: &'a str,
    context: String,
    primary: String,
    secondary: String,
    meta: Option<String>,
    preview: Option<String>,
    color: Color,
}

fn run_state_card(app: &App) -> Option<RunStateCard<'static>> {
    if app.status.agent_state == AgentStateDisplay::ToolExec {
        if let Some(tool) = latest_running_tool(app) {
            return Some(RunStateCard {
                title: " Running tool ",
                context: operator_context(app, "tool"),
                primary: tool.name.clone(),
                secondary: tool.detail.clone(),
                meta: None,
                preview: None,
                color: Color::Cyan,
            });
        }
    }
    if app.status.agent_state != AgentStateDisplay::Idle {
        return app
            .last_submitted_input
            .as_ref()
            .map(|prompt| RunStateCard {
                title: active_title(app.status.agent_state),
                context: operator_context(app, state_label(app.status.agent_state)),
                primary: prompt.clone(),
                secondary: active_turn_summary(app),
                meta: operator_meta(app),
                preview: streaming_preview(app),
                color: active_color(app.status.agent_state),
            });
    }
    app.last_submitted_input
        .as_ref()
        .map(|prompt| RunStateCard {
            title: " Last turn ",
            context: operator_context(app, "completed"),
            primary: prompt.clone(),
            secondary: completed_turn_summary(),
            meta: operator_meta(app),
            preview: None,
            color: Color::Green,
        })
}

fn active_turn_summary(app: &App) -> String {
    match app.status.agent_state {
        AgentStateDisplay::Thinking => "planning the next action".to_string(),
        AgentStateDisplay::ToolExec => "executing tool work".to_string(),
        AgentStateDisplay::Streaming => "assistant output arriving".to_string(),
        AgentStateDisplay::Error => app
            .last_operator_state
            .as_ref()
            .and_then(|state| state.reason_code.clone())
            .unwrap_or_else(|| "turn needs intervention".to_string()),
        AgentStateDisplay::Idle => "ready".to_string(),
    }
}

fn completed_turn_summary() -> String {
    "assistant reply ready".to_string()
}

fn state_label(state: AgentStateDisplay) -> &'static str {
    match state {
        AgentStateDisplay::Idle => "idle",
        AgentStateDisplay::Thinking => "thinking",
        AgentStateDisplay::ToolExec => "tool",
        AgentStateDisplay::Streaming => "streaming",
        AgentStateDisplay::Error => "error",
    }
}

fn active_title(state: AgentStateDisplay) -> &'static str {
    match state {
        AgentStateDisplay::Thinking => " Thinking ",
        AgentStateDisplay::ToolExec => " Running tool ",
        AgentStateDisplay::Streaming => " Streaming reply ",
        AgentStateDisplay::Error => " Turn failed ",
        AgentStateDisplay::Idle => " Last turn ",
    }
}

fn active_color(state: AgentStateDisplay) -> Color {
    match state {
        AgentStateDisplay::Thinking => Color::Yellow,
        AgentStateDisplay::ToolExec => Color::Cyan,
        AgentStateDisplay::Streaming => Color::Green,
        AgentStateDisplay::Error => Color::LightRed,
        AgentStateDisplay::Idle => Color::Green,
    }
}

fn operator_context(app: &App, fallback: &str) -> String {
    let Some(state) = &app.last_operator_state else {
        return fallback.to_string();
    };
    let phase = state.phase.as_deref().unwrap_or(state.status.as_str());
    format!("{}:{phase}", state.entity)
}

fn operator_meta(app: &App) -> Option<String> {
    let state = app.last_operator_state.as_ref()?;
    let mut parts = Vec::new();
    if let Some(kind) = &state.artifact_kind {
        parts.push(kind.clone());
    }
    if let Some(response_id) = &state.response_id {
        parts.push(response_id.clone());
    }
    if let Some(reason_code) = &state.reason_code {
        parts.push(reason_code.clone());
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join(" · "))
}

fn streaming_preview(app: &App) -> Option<String> {
    if app.status.agent_state != AgentStateDisplay::Streaming {
        return None;
    }
    app.chat
        .messages()
        .iter()
        .rev()
        .find(|msg| matches!(msg.role, super::super::chat::MessageRole::Assistant))
        .map(|msg| truncate(&msg.content, 72))
}

fn extend_with_meta(
    mut lines: Vec<Line<'static>>,
    meta: Option<String>,
) -> Vec<Line<'static>> {
    if let Some(text) = meta {
        lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines
}

fn extend_with_preview(
    mut lines: Vec<Line<'static>>,
    preview: Option<String>,
) -> Vec<Line<'static>> {
    if let Some(text) = preview {
        lines.push(Line::from(Span::styled(
            text,
            Style::default().fg(Color::LightCyan),
        )));
    }
    lines
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        return input.to_string();
    }
    if max <= 3 {
        return input[..max].to_string();
    }
    format!("{}...", &input[..max - 3])
}
