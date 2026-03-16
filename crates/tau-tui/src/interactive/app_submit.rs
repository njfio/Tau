use super::App;
use super::super::chat::{ChatMessage, MessageRole};
use super::super::tools::{ToolEntry, ToolStatus};
use super::super::status::AgentStateDisplay;

impl App {
    pub(super) fn submit_input(&mut self) {
        let text = self.input.get_text();
        if text.trim().is_empty() {
            return;
        }
        if text.starts_with('/') {
            let cmd = text.trim_start_matches('/');
            self.execute_command(cmd);
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
        self.push_message(
            MessageRole::Assistant,
            format!(
                "Received your message. (Model: {}, {} chars)",
                self.config.model,
                text.len()
            ),
        );
        self.status.total_messages += 2;
        self.status.total_tokens += text.len() as u64 / 4;
        self.chat.scroll_to_bottom();
        self.status.agent_state = AgentStateDisplay::Idle;
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
