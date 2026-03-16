use super::app::{timestamp, App};
use super::chat::MessageRole;
use super::status::AgentStateDisplay;

impl App {
    pub fn execute_command(&mut self, cmd: &str) {
        match cmd.trim() {
            "quit" | "q" => self.should_quit = true,
            "clear" => self.chat.clear(),
            "help" => self.show_help = true,
            "tools" => self.show_tool_panel = !self.show_tool_panel,
            _ => self.push_unknown_command(cmd),
        }
    }

    pub fn submit_input(&mut self) {
        let text = self.input.get_text();
        if text.trim().is_empty() {
            return;
        }
        if let Some(command) = text.strip_prefix('/') {
            self.execute_command(command);
            self.input.clear();
            return;
        }

        self.push_user_message(&text);
        self.input.clear();
        if self.gateway_runtime.is_some() {
            self.submit_gateway_prompt(text);
            return;
        }
        self.push_local_echo(text);
    }

    fn push_unknown_command(&mut self, cmd: &str) {
        self.push_message(MessageRole::System, format!("Unknown command: {cmd}"));
    }

    fn push_user_message(&mut self, text: &str) {
        self.chat.add_message(super::chat::ChatMessage {
            role: MessageRole::User,
            content: text.to_string(),
            timestamp: timestamp(),
        });
        self.status.total_messages += 1;
        self.status.total_tokens += text.len() as u64 / 4;
        self.chat.scroll_to_bottom();
    }

    fn submit_gateway_prompt(&mut self, text: String) {
        self.pending_assistant.clear();
        self.push_message(MessageRole::Assistant, String::new());
        self.status.agent_state = AgentStateDisplay::Thinking;
        let Some(runtime) = &self.gateway_runtime else {
            return;
        };
        if runtime.submit(text).is_err() {
            self.push_message(
                MessageRole::System,
                "gateway error: request channel closed".to_string(),
            );
            self.status.agent_state = AgentStateDisplay::Error;
        }
    }

    fn push_local_echo(&mut self, text: String) {
        self.push_message(
            MessageRole::Assistant,
            format!(
                "Received your message. (Model: {}, {} chars)",
                self.config.model,
                text.len()
            ),
        );
        self.status.total_messages += 1;
    }
}
