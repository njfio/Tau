//! Core application state for the interactive TUI.

use std::sync::mpsc::{Receiver, TryRecvError};

use crossterm::event::KeyEvent;

use super::chat::{ChatMessage, ChatPanel, MessageRole};
use super::gateway_client::{
    spawn_gateway_turn, GatewayRuntimeConfig, GatewayTurnResponse, GatewayTurnResult,
};
use super::input::InputEditor;
use super::status::{AgentStateDisplay, StatusBar};
use super::tools::{ToolEntry, ToolPanel, ToolStatus};

/// Configuration for the interactive TUI application.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model: String,
    pub profile: String,
    pub tick_rate_ms: u64,
    pub gateway: GatewayRuntimeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "gpt-5.3-codex".to_string(),
            profile: "local-dev".to_string(),
            tick_rate_ms: 100,
            gateway: GatewayRuntimeConfig::default(),
        }
    }
}

/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Chat,
    Input,
    Tools,
    CommandPalette,
}

/// Input mode for the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

/// Main application state.
pub struct App {
    pub config: AppConfig,
    pub chat: ChatPanel,
    pub input: InputEditor,
    pub status: StatusBar,
    pub tools: ToolPanel,
    pub focus: FocusPanel,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub show_help: bool,
    pub command_input: String,
    pub show_tool_panel: bool,
    current_turn_tool_start: usize,
    pending_turn: Option<Receiver<GatewayTurnResponse>>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let status = StatusBar::new(config.model.clone(), config.profile.clone());
        Self {
            config,
            chat: ChatPanel::new(),
            input: InputEditor::new(),
            status,
            tools: ToolPanel::new(),
            focus: FocusPanel::Input,
            input_mode: InputMode::Insert,
            should_quit: false,
            show_help: false,
            command_input: String::new(),
            show_tool_panel: true,
            current_turn_tool_start: 0,
            pending_turn: None,
        }
    }

    /// Process a key event and update app state.
    pub fn handle_key(&mut self, key: KeyEvent) {
        super::app_keys::handle_key(self, key);
    }

    pub fn tick(&mut self) {
        let Some(receiver) = self.pending_turn.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => self.finish_turn(result),
            Err(TryRecvError::Empty) => self.pending_turn = Some(receiver),
            Err(TryRecvError::Disconnected) => {
                self.fail_turn("gateway error: runtime worker disconnected".to_string());
            }
        }
    }

    pub fn submit_prompt(&mut self, prompt: String) {
        if self.pending_turn.is_some() {
            self.push_timestamped_message(
                MessageRole::System,
                "A turn is already in progress.".to_string(),
            );
            return;
        }

        self.status.agent_state = AgentStateDisplay::Thinking;
        self.status.total_messages += 1;
        self.push_message(MessageRole::User, prompt.clone());
        self.pending_turn = Some(spawn_gateway_turn(self.config.gateway.clone(), prompt));
    }

    /// Push a chat message externally (for agent integration).
    pub fn push_message(&mut self, role: MessageRole, content: String) {
        if role == MessageRole::User {
            self.start_turn();
        }
        self.push_timestamped_message(role, content);
    }

    pub fn current_turn_tools(&self) -> &[ToolEntry] {
        let start = self.current_turn_tool_start.min(self.tools.entries().len());
        &self.tools.entries()[start..]
    }

    pub fn latest_user_prompt(&self) -> Option<&str> {
        self.chat
            .messages()
            .iter()
            .rev()
            .find(|message| message.role == MessageRole::User)
            .map(|message| message.content.as_str())
    }

    /// Push a tool execution event externally.
    pub fn push_tool_event(&mut self, name: String, status: ToolStatus, detail: String) {
        self.tools.add_entry(ToolEntry {
            name,
            status,
            detail,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }

    fn start_turn(&mut self) {
        self.current_turn_tool_start = self.tools.total_count();
    }

    fn finish_turn(&mut self, result: GatewayTurnResponse) {
        match result {
            Ok(result) => self.complete_turn(result),
            Err(error) => self.fail_turn(format!("gateway error: {error}")),
        }
    }

    fn complete_turn(&mut self, result: GatewayTurnResult) {
        self.status.agent_state = AgentStateDisplay::Idle;
        self.status.total_messages += 1;
        self.status.total_tokens = self.status.total_tokens.saturating_add(result.total_tokens);
        self.push_timestamped_message(MessageRole::Assistant, result.output_text);
    }

    fn fail_turn(&mut self, message: String) {
        self.status.agent_state = AgentStateDisplay::Error;
        self.push_timestamped_message(MessageRole::System, message);
    }

    fn push_timestamped_message(&mut self, role: MessageRole, content: String) {
        self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.chat.scroll_to_bottom();
    }
}
