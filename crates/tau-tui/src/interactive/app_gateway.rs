use std::mem;

use chrono::Local;

use super::{
    app::App,
    chat::MessageRole,
    gateway::{GatewayUiEvent, OperatorStateEvent},
    status::AgentStateDisplay,
    tools::ToolStatus,
};

impl App {
    pub fn apply_gateway_event(&mut self, event: GatewayUiEvent) {
        match event {
            GatewayUiEvent::OperatorState(state) => self.apply_operator_state(state),
            GatewayUiEvent::AssistantDelta(delta) => self.pending_assistant.push_str(&delta),
            GatewayUiEvent::AssistantDone(text) => self.finish_assistant_text(text),
            GatewayUiEvent::ResponseCompleted(text) => self.finish_completed_turn(text),
            GatewayUiEvent::Failure(message) => self.push_gateway_failure(message),
        }
    }

    fn apply_operator_state(&mut self, state: OperatorStateEvent) {
        self.status.agent_state = agent_state_for(&state);
        self.push_tool_event(
            state.entity.clone(),
            tool_status_for(&state.status),
            state_detail(&state),
        );
    }

    fn finish_completed_turn(&mut self, text: Option<String>) {
        if !self.pending_assistant.is_empty() || text.is_some() {
            let final_text = text.unwrap_or_else(|| mem::take(&mut self.pending_assistant));
            self.finish_assistant_text(final_text);
        }
        if self.status.agent_state != AgentStateDisplay::Error {
            self.status.agent_state = AgentStateDisplay::Idle;
        }
    }

    fn finish_assistant_text(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        self.pending_assistant.clear();
        self.push_message(MessageRole::Assistant, text.clone());
        self.status.total_messages += 1;
        self.status.total_tokens += (text.len() / 4) as u64;
        self.status.agent_state = AgentStateDisplay::Streaming;
    }

    fn push_gateway_failure(&mut self, message: String) {
        self.push_message(MessageRole::System, format!("gateway error: {message}"));
        self.status.agent_state = AgentStateDisplay::Error;
    }
}

fn tool_status_for(status: &str) -> ToolStatus {
    match status {
        "failed" => ToolStatus::Failed,
        "completed" => ToolStatus::Success,
        _ => ToolStatus::Running,
    }
}

fn agent_state_for(state: &OperatorStateEvent) -> AgentStateDisplay {
    if state.status == "failed" {
        return AgentStateDisplay::Error;
    }
    if state.entity == "artifact" && state.status == "streaming" {
        return AgentStateDisplay::Streaming;
    }
    if state.entity == "tool" && state.status == "in_progress" {
        return AgentStateDisplay::ToolExec;
    }
    if state.entity == "turn" && state.status == "completed" {
        return AgentStateDisplay::Idle;
    }
    AgentStateDisplay::Thinking
}

fn state_detail(state: &OperatorStateEvent) -> String {
    let mut parts = vec![format!("status={}", state.status)];
    if let Some(phase) = &state.phase {
        parts.push(format!("phase={phase}"));
    }
    if let Some(kind) = &state.artifact_kind {
        parts.push(format!("artifact_kind={kind}"));
    }
    if let Some(code) = &state.reason_code {
        parts.push(format!("reason_code={code}"));
    }
    parts.push(format!("at={}", Local::now().format("%H:%M:%S")));
    parts.join(" ")
}
