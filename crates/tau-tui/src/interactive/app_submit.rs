use super::super::chat::{ChatMessage, MessageRole};
use super::super::status::AgentStateDisplay;
use super::super::tools::{ToolEntry, ToolStatus};
use super::App;

impl App {
    pub(crate) fn submit_input(&mut self) {
        let text = self.input.get_text();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        if let Some(cmd) = slash_command(trimmed) {
            self.execute_command(cmd);
            self.input.clear();
            return;
        }
        if is_inline_command(trimmed) {
            self.execute_command(trimmed);
            self.input.clear();
            return;
        }
        self.last_submitted_input = Some(text.clone());
        self.run_prompt(text);
        self.input.clear();
    }

    pub(super) fn retry_last_prompt(&mut self) {
        let Some(prompt) = self.last_submitted_input.clone() else {
            self.push_system_note("No previous prompt available to retry.");
            return;
        };
        self.push_system_note("Retrying last prompt.");
        self.run_prompt(prompt);
    }

    fn run_prompt(&mut self, text: String) {
        self.status.agent_state = AgentStateDisplay::Thinking;
        self.push_message(MessageRole::User, text.clone());
        self.status.total_messages += 1;
        self.status.total_tokens += text.len() as u64 / 4;
        self.chat.scroll_to_bottom();
        if self.submit_gateway_prompt(text.clone()) {
            return;
        }
        self.push_message(
            MessageRole::Assistant,
            format!(
                "Received your message. (Model: {}, {} chars)",
                self.config.model,
                text.len()
            ),
        );
        self.status.total_messages += 1;
        self.chat.scroll_to_bottom();
        self.status.agent_state = AgentStateDisplay::Idle;
    }

    fn submit_gateway_prompt(&mut self, text: String) -> bool {
        if self.gateway_runtime.is_none() {
            return false;
        }
        self.pending_assistant.clear();
        self.push_message(MessageRole::Assistant, String::new());
        self.status.total_messages += 1;
        let runtime = self
            .gateway_runtime
            .as_ref()
            .expect("gateway runtime checked above");
        if runtime.submit(text).is_ok() {
            return true;
        }
        self.push_message(
            MessageRole::System,
            "gateway error: request channel closed".to_string(),
        );
        self.status.total_messages += 1;
        self.status.agent_state = AgentStateDisplay::Error;
        true
    }

    pub(super) fn push_system_note(&mut self, content: &str) {
        self.push_message(MessageRole::System, content.to_string());
    }

    pub fn push_message(&mut self, role: MessageRole, content: String) {
        self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.chat.scroll_to_bottom();
    }

    pub fn push_tool_event(&mut self, name: String, status: ToolStatus, detail: String) {
        self.tools.add_entry(ToolEntry {
            name,
            status,
            detail,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }
}

fn is_inline_command(text: &str) -> bool {
    matches!(
        text,
        "quit"
            | "q"
            | "clear"
            | "help"
            | "thinking"
            | "details"
            | "tools"
            | "memory"
            | "cortex"
            | "sessions"
            | "approval-needed"
            | "approve"
            | "reject"
            | "interrupt"
            | "retry"
    )
}

fn slash_command(text: &str) -> Option<&str> {
    text.strip_prefix('/').map(str::trim)
}
