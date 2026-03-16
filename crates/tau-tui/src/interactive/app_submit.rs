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
        if self.config.gateway.is_some() {
            self.enter_gateway_wait_state();
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

    fn enter_gateway_wait_state(&mut self) {
        self.pending_assistant.clear();
        self.status.agent_state = AgentStateDisplay::Thinking;
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
