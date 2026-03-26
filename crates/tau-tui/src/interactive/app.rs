//! Core application state for the interactive TUI.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

use crossterm::event::KeyEvent;
use tau_skills::{load_catalogs, select_skills_for_prompt};

use super::chat::{ChatMessage, ChatPanel, MessageRole};
use super::gateway_client::{
    spawn_gateway_turn_streaming, GatewayRuntimeConfig, GatewayStreamEvent, GatewayTurnResult,
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
    pub skills_dir: PathBuf,
    pub bundled_skills_dir: Option<PathBuf>,
    pub gateway: GatewayRuntimeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "gpt-5.3-codex".to_string(),
            profile: "local-dev".to_string(),
            tick_rate_ms: 100,
            skills_dir: PathBuf::from(".tau/skills"),
            bundled_skills_dir: Some(PathBuf::from("skills")),
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
    pending_stream: Option<Receiver<GatewayStreamEvent>>,
    streaming_text: String,
    streaming_message_index: Option<usize>,
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
            pending_stream: None,
            streaming_text: String::new(),
            streaming_message_index: None,
        }
    }

    /// Process a key event and update app state.
    pub fn handle_key(&mut self, key: KeyEvent) {
        super::app_keys::handle_key(self, key);
    }

    pub fn tick(&mut self) {
        let Some(receiver) = self.pending_stream.take() else {
            return;
        };

        // Drain all available events this tick for responsive streaming
        loop {
            match receiver.try_recv() {
                Ok(GatewayStreamEvent::Delta(delta)) => {
                    self.status.agent_state = AgentStateDisplay::Streaming;
                    self.streaming_text.push_str(&delta);
                    if let Some(idx) = self.streaming_message_index {
                        self.chat.update_message_content(idx, self.streaming_text.clone());
                    } else {
                        let idx = self.chat.add_message(ChatMessage {
                            role: MessageRole::Assistant,
                            content: self.streaming_text.clone(),
                            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        });
                        self.streaming_message_index = Some(idx);
                    }
                    self.chat.scroll_to_bottom();
                }
                Ok(GatewayStreamEvent::UsageUpdate { total_tokens, .. }) => {
                    self.status.total_tokens = self.status.total_tokens.saturating_add(total_tokens);
                }
                Ok(GatewayStreamEvent::CostUpdate { cumulative_cost_cents }) => {
                    self.status.total_cost_cents = cumulative_cost_cents;
                }
                Ok(GatewayStreamEvent::ToolStart { tool_name, arguments_preview }) => {
                    self.status.agent_state = AgentStateDisplay::ToolExec;
                    self.tools.add_entry(ToolEntry {
                        name: tool_name,
                        status: ToolStatus::Running,
                        detail: arguments_preview,
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    });
                }
                Ok(GatewayStreamEvent::MessageAdded { role, text }) => {
                    // Only show assistant messages in chat — hide internal user/system nudges
                    if role == "assistant" && !text.trim().is_empty() {
                        self.streaming_text.push_str(&text);
                        self.streaming_text.push('\n');
                        if let Some(idx) = self.streaming_message_index {
                            self.chat.update_message_content(idx, self.streaming_text.clone());
                        } else {
                            let idx = self.chat.add_message(ChatMessage {
                                role: MessageRole::Assistant,
                                content: self.streaming_text.clone(),
                                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                            });
                            self.streaming_message_index = Some(idx);
                        }
                    } else {
                        // Don't show user/system nudge messages — they're internal to the loop
                    }
                    self.chat.scroll_to_bottom();
                }
                Ok(GatewayStreamEvent::ToolEnd { tool_name, success, output_preview }) => {
                    self.status.agent_state = AgentStateDisplay::Streaming;
                    self.tools.add_entry(ToolEntry {
                        name: tool_name,
                        status: if success { ToolStatus::Success } else { ToolStatus::Failed },
                        detail: output_preview,
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    });
                }
                Ok(GatewayStreamEvent::Done(result)) => {
                    self.complete_streaming_turn(result);
                    return; // Stream is done, don't put receiver back
                }
                Ok(GatewayStreamEvent::Error(error)) => {
                    self.finish_streaming();
                    self.fail_turn(format!("gateway error: {error}"));
                    return;
                }
                Err(TryRecvError::Empty) => {
                    // No more events right now — put receiver back and wait for next tick
                    self.pending_stream = Some(receiver);
                    return;
                }
                Err(TryRecvError::Disconnected) => {
                    self.finish_streaming();
                    self.fail_turn("gateway error: runtime worker disconnected".to_string());
                    return;
                }
            }
        }
    }

    pub fn submit_prompt(&mut self, prompt: String) {
        if self.pending_stream.is_some() {
            self.push_timestamped_message(
                MessageRole::System,
                "A turn is already in progress.".to_string(),
            );
            return;
        }

        if let Err(error) = self.update_active_skills_for_prompt(&prompt) {
            self.status.active_skills.clear();
            self.push_timestamped_message(
                MessageRole::System,
                format!("skill selection warning: {error}"),
            );
        }
        self.status.agent_state = AgentStateDisplay::Thinking;
        self.status.total_messages += 1;
        self.push_message(MessageRole::User, prompt.clone());
        self.start_turn();
        self.pending_stream = Some(spawn_gateway_turn_streaming(
            self.config.gateway.clone(),
            prompt,
        ));
    }

    pub fn update_active_skills_for_prompt(&mut self, prompt: &str) -> Result<(), String> {
        let mut dirs = vec![self.config.skills_dir.clone()];
        if let Some(dir) = &self.config.bundled_skills_dir {
            if dir != &self.config.skills_dir {
                dirs.push(dir.clone());
            }
        }
        let catalog = load_catalogs(&dirs).map_err(|error| format!("skill catalog: {error}"))?;
        let report = select_skills_for_prompt(&catalog, &[], prompt)
            .map_err(|error| format!("skill selection: {error}"))?;
        self.status.active_skills = report
            .selected
            .iter()
            .map(|skill| skill.name.clone())
            .collect::<Vec<_>>();
        Ok(())
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
        self.streaming_text.clear();
        self.streaming_message_index = None;
    }

    fn finish_streaming(&mut self) {
        self.streaming_text.clear();
        self.streaming_message_index = None;
    }

    fn complete_streaming_turn(&mut self, result: GatewayTurnResult) {
        self.status.agent_state = AgentStateDisplay::Idle;
        self.status.total_messages += 1;
        self.status.total_tokens = self.status.total_tokens.saturating_add(result.total_tokens);

        // If we already have a streaming message, update it with final text
        if let Some(idx) = self.streaming_message_index {
            if !result.output_text.is_empty() {
                self.chat.update_message_content(idx, result.output_text);
            }
        } else if !result.output_text.is_empty() {
            // No streaming happened — show the full response
            self.push_timestamped_message(MessageRole::Assistant, result.output_text);
        }
        self.finish_streaming();
    }

    fn fail_turn(&mut self, message: String) {
        self.status.agent_state = AgentStateDisplay::Error;
        self.push_timestamped_message(MessageRole::System, message);
    }

    pub(crate) fn push_timestamped_message(&mut self, role: MessageRole, content: String) {
        self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.chat.scroll_to_bottom();
    }
}
