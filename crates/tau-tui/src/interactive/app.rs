//! Core interactive application state.

#[path = "app_commands.rs"]
mod commands;
#[path = "app_detail.rs"]
mod detail;
#[path = "app_focus.rs"]
mod focus;
#[path = "app_nav.rs"]
mod nav;
#[path = "app_runtime.rs"]
pub mod runtime;
#[path = "app_submit.rs"]
mod submit;

use super::chat::ChatPanel;
use super::gateway::OperatorStateEvent;
use super::input::InputEditor;
use super::status::StatusBar;
use super::tools::ToolPanel;
use super::{gateway_runtime::GatewayRuntime, GatewayInteractiveConfig};

pub use runtime::run_interactive;

pub const LOCAL_TUI_DEFAULT_MODEL: &str = "gpt-5.2-codex";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model: String,
    pub profile: String,
    pub session_key: String,
    pub workspace_label: String,
    pub approval_mode: String,
    pub tick_rate_ms: u64,
    pub gateway: Option<GatewayInteractiveConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: LOCAL_TUI_DEFAULT_MODEL.to_string(),
            profile: "local-dev".to_string(),
            session_key: "default".to_string(),
            workspace_label: ".".to_string(),
            approval_mode: "ask".to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSection {
    Tools,
    Memory,
    Cortex,
    Sessions,
}

impl DetailSection {
    pub fn label(self) -> &'static str {
        match self {
            DetailSection::Tools => "tools",
            DetailSection::Memory => "memory",
            DetailSection::Cortex => "cortex",
            DetailSection::Sessions => "sessions",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest {
    pub summary: String,
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
    pub show_thinking: bool,
    pub command_input: String,
    pub show_tool_panel: bool,
    pub detail_section: DetailSection,
    pub approval_request: Option<ApprovalRequest>,
    pub last_submitted_input: Option<String>,
    pub pending_assistant: String,
    pub gateway_runtime: Option<GatewayRuntime>,
    pub last_operator_state: Option<OperatorStateEvent>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let status = StatusBar::new(config.model.clone(), config.profile.clone());
        let gateway_runtime = config.gateway.clone().map(GatewayRuntime::start);
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
            show_thinking: false,
            command_input: String::new(),
            show_tool_panel: false,
            detail_section: DetailSection::Tools,
            approval_request: None,
            last_submitted_input: None,
            pending_assistant: String::new(),
            gateway_runtime,
            last_operator_state: None,
        }
    }
}
