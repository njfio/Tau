//! Core application state for the interactive TUI.

use crossterm::event::KeyEvent;

use super::chat::{ChatMessage, ChatPanel, MessageRole};
use super::input::InputEditor;
use super::status::StatusBar;
use super::tools::{ToolEntry, ToolPanel, ToolStatus};

/// Configuration for the interactive TUI application.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model: String,
    pub profile: String,
    pub tick_rate_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "openai/gpt-5.2".to_string(),
            profile: "local-dev".to_string(),
            tick_rate_ms: 100,
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
        }
    }

    /// Process a key event and update app state.
    pub fn handle_key(&mut self, key: KeyEvent) {
        super::app_keys::handle_key(self, key);
    }

    /// Push a chat message externally (for agent integration).
    pub fn push_message(&mut self, role: MessageRole, content: String) {
        self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.chat.scroll_to_bottom();
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
}
