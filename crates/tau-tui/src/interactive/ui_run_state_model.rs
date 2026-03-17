use ratatui::style::Color;

use crate::interactive::app::App;
use crate::interactive::chat::MessageRole;
use crate::interactive::status::AgentStateDisplay;

use super::super::shared::latest_running_tool;

pub(super) struct RunStateCard<'a> {
    pub(super) title: &'a str,
    pub(super) context: String,
    pub(super) primary: String,
    pub(super) secondary: String,
    pub(super) meta: Option<String>,
    pub(super) preview: Option<String>,
    pub(super) color: Color,
}

pub(super) fn build_run_state_card(app: &App) -> Option<RunStateCard<'static>> {
    tool_state_card(app)
        .or_else(|| active_turn_card(app))
        .or_else(|| completed_turn_card(app))
}

fn tool_state_card(app: &App) -> Option<RunStateCard<'static>> {
    if app.status.agent_state != AgentStateDisplay::ToolExec {
        return None;
    }
    let tool = latest_running_tool(app)?;
    Some(RunStateCard {
        title: " Running tool ",
        context: operator_context(app, "tool"),
        primary: tool.name.clone(),
        secondary: tool.detail.clone(),
        meta: None,
        preview: None,
        color: Color::Cyan,
    })
}

fn active_turn_card(app: &App) -> Option<RunStateCard<'static>> {
    if app.status.agent_state == AgentStateDisplay::Idle {
        return None;
    }
    let prompt = app.last_submitted_input.as_ref()?;
    Some(RunStateCard {
        title: active_title(app.status.agent_state),
        context: operator_context(app, state_label(app.status.agent_state)),
        primary: prompt.clone(),
        secondary: active_turn_summary(app),
        meta: operator_meta(app),
        preview: streaming_preview(app),
        color: active_color(app.status.agent_state),
    })
}

fn completed_turn_card(app: &App) -> Option<RunStateCard<'static>> {
    let prompt = app.last_submitted_input.as_ref()?;
    Some(RunStateCard {
        title: " Last turn ",
        context: operator_context(app, "completed"),
        primary: prompt.clone(),
        secondary: "assistant reply ready".to_string(),
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
        AgentStateDisplay::Error => failure_summary(app),
        AgentStateDisplay::Idle => "ready".to_string(),
    }
}

fn failure_summary(app: &App) -> String {
    app.last_operator_state
        .as_ref()
        .and_then(|state| state.reason_code.clone())
        .unwrap_or_else(|| "turn needs intervention".to_string())
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

fn state_label(state: AgentStateDisplay) -> &'static str {
    match state {
        AgentStateDisplay::Idle => "idle",
        AgentStateDisplay::Thinking => "thinking",
        AgentStateDisplay::ToolExec => "tool",
        AgentStateDisplay::Streaming => "streaming",
        AgentStateDisplay::Error => "error",
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
        .find(|msg| matches!(msg.role, MessageRole::Assistant))
        .map(|msg| truncate(&msg.content, 72))
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
