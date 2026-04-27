use tau_contract::operator_state::{
    OperatorToolState, OperatorToolStatus, OperatorTurnPhase, OperatorTurnState, OperatorTurnStatus,
};

use super::{
    app::App,
    chat::{ChatMessage, MessageRole},
    status::AgentStateDisplay,
    tools::{ToolEntry, ToolStatus},
};

pub fn apply_operator_turn_state(app: &mut App, state: &OperatorTurnState) {
    bind_operator_turn_identity(app, state);
    apply_operator_tools(app, &state.tools);
    apply_assistant_text(app, state.assistant_text.as_str());
    apply_operator_status(app, state);
}

fn bind_operator_turn_identity(app: &mut App, state: &OperatorTurnState) {
    app.config.gateway.session_key = state.session_key.clone();
    app.config.gateway.mission_id = state.mission_id.clone();
    app.status.active_mission_id = state.mission_id.clone();
}

fn apply_assistant_text(app: &mut App, assistant_text: &str) {
    if assistant_text.trim().is_empty() {
        return;
    }
    if app.chat.last_assistant_content() == Some(assistant_text) {
        return;
    }
    app.chat.add_message(ChatMessage {
        role: MessageRole::Assistant,
        content: assistant_text.to_string(),
        timestamp: now_timestamp(),
    });
    app.chat.scroll_to_bottom();
}

fn apply_operator_tools(app: &mut App, tools: &[OperatorToolState]) {
    for tool in tools {
        match map_tool_status(&tool.status) {
            ToolStatus::Running => ensure_running_tool(app, tool),
            status => complete_or_add_tool(app, tool, status),
        }
    }
}

fn ensure_running_tool(app: &mut App, tool: &OperatorToolState) {
    if has_tool_entry(app, tool.tool_call_id.as_str()) {
        return;
    }
    app.tools.add_entry(ToolEntry {
        tool_call_id: Some(tool.tool_call_id.clone()),
        name: tool.tool_name.clone(),
        status: ToolStatus::Running,
        detail: tool_detail(tool),
        timestamp: now_timestamp(),
    });
}

fn complete_or_add_tool(app: &mut App, tool: &OperatorToolState, status: ToolStatus) {
    if app
        .tools
        .complete_running_by_id(tool.tool_call_id.as_str(), status, tool_detail(tool))
    {
        return;
    }
    if has_tool_entry(app, tool.tool_call_id.as_str()) {
        return;
    }
    app.tools.add_entry(ToolEntry {
        tool_call_id: Some(tool.tool_call_id.clone()),
        name: tool.tool_name.clone(),
        status,
        detail: tool_detail(tool),
        timestamp: now_timestamp(),
    });
}

fn has_tool_entry(app: &App, tool_call_id: &str) -> bool {
    app.tools
        .entries()
        .iter()
        .any(|entry| entry.tool_call_id.as_deref() == Some(tool_call_id))
}

fn tool_detail(tool: &OperatorToolState) -> String {
    tool.summary.clone().unwrap_or_default()
}

fn map_tool_status(status: &OperatorToolStatus) -> ToolStatus {
    match status {
        OperatorToolStatus::Pending | OperatorToolStatus::Running => ToolStatus::Running,
        OperatorToolStatus::Completed => ToolStatus::Success,
        OperatorToolStatus::Failed => ToolStatus::Failed,
        OperatorToolStatus::Cancelled => ToolStatus::Failed,
    }
}

fn apply_operator_status(app: &mut App, state: &OperatorTurnState) {
    app.status.agent_state = match (&state.status, &state.phase) {
        (OperatorTurnStatus::Succeeded, OperatorTurnPhase::Completed) => AgentStateDisplay::Idle,
        (OperatorTurnStatus::ToolRunning, _) | (_, OperatorTurnPhase::WaitingForTool) => {
            AgentStateDisplay::ToolExec
        }
        (OperatorTurnStatus::Streaming, _) => AgentStateDisplay::Streaming,
        (OperatorTurnStatus::Pending, _)
        | (_, OperatorTurnPhase::Queued | OperatorTurnPhase::Running) => {
            AgentStateDisplay::Thinking
        }
        (
            OperatorTurnStatus::Blocked
            | OperatorTurnStatus::TimedOut
            | OperatorTurnStatus::Failed
            | OperatorTurnStatus::Cancelled,
            _,
        ) => {
            push_operator_error_message(app, state);
            AgentStateDisplay::Error
        }
        _ => AgentStateDisplay::Thinking,
    };
}

fn push_operator_error_message(app: &mut App, state: &OperatorTurnState) {
    let event_summary = state
        .events
        .iter()
        .rev()
        .find_map(|event| (!event.summary.trim().is_empty()).then_some(event.summary.as_str()));
    let message = match state.error.as_ref() {
        Some(error) => match event_summary {
            Some(summary) => format!(
                "operator turn {}: {} - {}; {}",
                state.turn_id, error.reason_code, error.message, summary
            ),
            None => format!(
                "operator turn {}: {} - {}",
                state.turn_id, error.reason_code, error.message
            ),
        },
        None => format!("operator turn {}: {:?}", state.turn_id, state.status),
    };
    if app
        .chat
        .messages()
        .iter()
        .rev()
        .any(|entry| entry.role == MessageRole::System && entry.content == message)
    {
        return;
    }
    app.chat.add_message(ChatMessage {
        role: MessageRole::System,
        content: message,
        timestamp: now_timestamp(),
    });
    app.chat.scroll_to_bottom();
}

fn now_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}
