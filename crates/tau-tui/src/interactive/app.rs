//! Core interactive application state.

#[path = "app_commands.rs"]
mod commands;
#[path = "app_runtime.rs"]
pub mod runtime;
#[path = "app_submit.rs"]
mod submit;

use super::chat::ChatPanel;
use super::input::InputEditor;
use super::status::StatusBar;
use super::tools::ToolPanel;

pub use runtime::run_interactive;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model: String,
    pub profile: String,
    pub session_key: String,
    pub workspace_label: String,
    pub approval_mode: String,
    pub tick_rate_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "openai/gpt-5.2".to_string(),
            profile: "local-dev".to_string(),
            session_key: "default".to_string(),
            workspace_label: ".".to_string(),
            approval_mode: "ask".to_string(),
            tick_rate_ms: 100,
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
    pub last_submitted_input: Option<String>,
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
            show_tool_panel: false,
            last_submitted_input: None,
        }
    }
}
