//! Core interactive application state.

use chrono::Local;

use super::chat::{ChatMessage, ChatPanel, MessageRole};
use super::gateway::GatewayInteractiveConfig;
use super::input::InputEditor;
use super::status::StatusBar;
use super::tools::{ToolEntry, ToolPanel, ToolStatus};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model: String,
    pub profile: String,
    pub tick_rate_ms: u64,
    pub gateway: Option<GatewayInteractiveConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "openai/gpt-5.2".to_string(),
            profile: "local-dev".to_string(),
            tick_rate_ms: 100,
            gateway: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Chat,
    Input,
    Tools,
    CommandPalette,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

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
    pub pending_assistant: String,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            status: StatusBar::new(config.model.clone(), config.profile.clone()),
            config,
            chat: ChatPanel::new(),
            input: InputEditor::new(),
            tools: ToolPanel::new(),
            focus: FocusPanel::Input,
            input_mode: InputMode::Insert,
            should_quit: false,
            show_help: false,
            command_input: String::new(),
            show_tool_panel: true,
            pending_assistant: String::new(),
        }
    }

    pub fn push_message(&mut self, role: MessageRole, content: String) {
        self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: timestamp(),
        });
        self.chat.scroll_to_bottom();
    }

    pub fn push_tool_event(&mut self, name: String, status: ToolStatus, detail: String) {
        self.tools.add_entry(ToolEntry {
            name,
            status,
            detail,
            timestamp: timestamp(),
        });
    }
}

pub fn timestamp() -> String {
    Local::now().format("%H:%M:%S").to_string()
}
