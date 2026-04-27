//! Core application state for the interactive TUI.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

use crossterm::event::KeyEvent;
use tau_skills::{load_catalogs, select_skills_for_prompt};

use super::chat::{ChatMessage, ChatPanel, MessageRole};
use super::gateway_client::{
    fetch_gateway_mission_detail, fetch_gateway_missions, spawn_gateway_turn,
    GatewayMissionSnapshot, GatewayRuntimeConfig, GatewayToolStatus, GatewayTurnEvent,
    GatewayTurnResponse, GatewayTurnResult,
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
    pub command_cursor_col: usize,
    pub show_tool_panel: bool,
    mouse_captured: bool,
    current_turn_tool_start: usize,
    current_operator_turn_id: Option<String>,
    streaming_assistant_index: Option<usize>,
    pending_turn: Option<Receiver<GatewayTurnEvent>>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let mut status = StatusBar::new(config.model.clone(), config.profile.clone());
        status.active_mission_id = config.gateway.mission_id.clone();
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
            command_cursor_col: 0,
            show_tool_panel: true,
            mouse_captured: true,
            current_turn_tool_start: 0,
            current_operator_turn_id: None,
            streaming_assistant_index: None,
            pending_turn: None,
        }
    }

    pub fn list_missions(&mut self) {
        match fetch_gateway_missions(&self.config.gateway, 20) {
            Ok(missions) => {
                self.push_timestamped_message(
                    MessageRole::System,
                    render_mission_list(missions.as_slice()),
                );
            }
            Err(error) => {
                self.push_timestamped_message(
                    MessageRole::System,
                    format!("mission control error: {error}"),
                );
            }
        }
    }

    pub fn show_mission(&mut self, mission_id: &str) {
        match fetch_gateway_mission_detail(&self.config.gateway, mission_id) {
            Ok(mission) => {
                self.push_timestamped_message(MessageRole::System, render_mission_detail(&mission));
            }
            Err(error) => {
                self.push_timestamped_message(
                    MessageRole::System,
                    format!("mission control error: {error}"),
                );
            }
        }
    }

    pub fn resume_mission(&mut self, mission_id: &str) {
        match fetch_gateway_mission_detail(&self.config.gateway, mission_id) {
            Ok(mission) => {
                if !matches!(mission.status.as_str(), "checkpointed" | "blocked") {
                    self.push_timestamped_message(
                        MessageRole::System,
                        format!(
                            "mission control error: mission {} is {} and cannot be resumed from this surface",
                            mission.mission_id, mission.status
                        ),
                    );
                    return;
                }
                self.bind_active_mission(&mission);
                let mut content = format!(
                    "Resumed mission {} on session {}.\nstatus: {}\ngoal: {}",
                    mission.mission_id, mission.session_key, mission.status, mission.goal_summary
                );
                if let Some(completion) = mission.latest_completion.as_ref() {
                    content.push_str(&format!(
                        "\nlatest completion: {} - {}",
                        completion.status, completion.summary
                    ));
                    if let Some(next_step) = completion.next_step.as_deref() {
                        content.push_str(&format!("\nnext step: {next_step}"));
                    }
                }
                self.push_timestamped_message(MessageRole::System, content);
            }
            Err(error) => {
                self.push_timestamped_message(
                    MessageRole::System,
                    format!("mission control error: {error}"),
                );
            }
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

        let mut keep_receiver = true;
        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    if self.apply_turn_event(event) {
                        keep_receiver = false;
                        break;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.fail_turn("gateway error: runtime worker disconnected".to_string());
                    keep_receiver = false;
                    break;
                }
            }
        }

        if keep_receiver {
            self.pending_turn = Some(receiver);
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
        self.pending_turn = Some(spawn_gateway_turn(self.config.gateway.clone(), prompt));
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
        self.push_tool_event_with_id(None, name, status, detail);
    }

    fn push_tool_event_with_id(
        &mut self,
        tool_call_id: Option<String>,
        name: String,
        status: ToolStatus,
        detail: String,
    ) {
        self.tools.add_entry(ToolEntry {
            tool_call_id,
            name,
            status,
            detail,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
    }

    fn start_turn(&mut self) {
        self.current_turn_tool_start = self.tools.total_count();
        self.current_operator_turn_id = None;
        self.streaming_assistant_index = None;
    }

    fn accept_operator_turn_snapshot(&mut self, turn_id: &str) -> bool {
        if turn_id.trim().is_empty() {
            return true;
        }
        match self.current_operator_turn_id.as_deref() {
            Some(current_turn_id) => current_turn_id == turn_id,
            None => {
                self.current_operator_turn_id = Some(turn_id.to_string());
                true
            }
        }
    }

    fn apply_turn_event(&mut self, event: GatewayTurnEvent) -> bool {
        match event {
            GatewayTurnEvent::TurnStarted { turn_id } => {
                self.current_operator_turn_id = Some(turn_id);
                false
            }
            GatewayTurnEvent::TextDelta(delta) => {
                self.append_assistant_delta(&delta);
                false
            }
            GatewayTurnEvent::OperatorStateSnapshot(state) => {
                if self.accept_operator_turn_snapshot(state.turn_id.as_str()) {
                    super::operator_state::apply_operator_turn_state(self, &state);
                }
                false
            }
            GatewayTurnEvent::ToolStarted {
                tool_call_id,
                tool_name,
                detail,
            } => {
                self.status.agent_state = AgentStateDisplay::ToolExec;
                self.push_tool_event_with_id(
                    Some(tool_call_id),
                    tool_name,
                    ToolStatus::Running,
                    detail,
                );
                false
            }
            GatewayTurnEvent::ToolCompleted {
                tool_call_id,
                tool_name,
                status,
                detail,
            } => {
                self.complete_tool_event(
                    tool_call_id,
                    tool_name,
                    map_gateway_tool_status(status),
                    detail,
                );
                self.status.agent_state = AgentStateDisplay::Thinking;
                false
            }
            GatewayTurnEvent::Finished(result) => {
                self.finish_turn(result);
                true
            }
        }
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
        match self.streaming_assistant_index.take() {
            Some(index) => {
                let current = self.chat.message_content(index).unwrap_or_default();
                if current.trim().is_empty() {
                    let _ = self.chat.set_message_content(index, result.output_text);
                }
            }
            None => {
                if self.chat.last_assistant_content() != Some(result.output_text.as_str()) {
                    self.push_timestamped_message(MessageRole::Assistant, result.output_text);
                }
            }
        }
    }

    fn fail_turn(&mut self, message: String) {
        self.status.agent_state = AgentStateDisplay::Error;
        self.streaming_assistant_index = None;
        self.push_timestamped_message(MessageRole::System, message);
    }

    fn append_assistant_delta(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        if let Some(index) = self.streaming_assistant_index {
            if self.chat.append_to_message(index, delta) {
                self.chat.scroll_to_bottom();
                return;
            }
        }
        let index = self.push_timestamped_message(MessageRole::Assistant, String::new());
        self.streaming_assistant_index = Some(index);
        let _ = self.chat.append_to_message(index, delta);
        self.chat.scroll_to_bottom();
    }

    fn complete_tool_event(
        &mut self,
        tool_call_id: String,
        name: String,
        status: ToolStatus,
        detail: String,
    ) {
        if !self
            .tools
            .complete_running_by_id(&tool_call_id, status, detail.clone())
            && !self
                .tools
                .complete_latest_running(&name, status, detail.clone())
        {
            self.push_tool_event(name, status, detail);
        }
    }

    fn push_timestamped_message(&mut self, role: MessageRole, content: String) -> usize {
        let index = self.chat.add_message(ChatMessage {
            role,
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.chat.scroll_to_bottom();
        index
    }

    pub fn toggle_mouse_capture(&mut self) {
        use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
        use crossterm::execute;
        if self.mouse_captured {
            let _ = execute!(std::io::stdout(), DisableMouseCapture);
            self.mouse_captured = false;
            self.push_timestamped_message(
                MessageRole::System,
                "Mouse capture disabled. Use /toggle-mouse or Ctrl+M to re-enable.".to_string(),
            );
        } else {
            let _ = execute!(std::io::stdout(), EnableMouseCapture);
            self.mouse_captured = true;
            self.push_timestamped_message(
                MessageRole::System,
                "Mouse capture enabled.".to_string(),
            );
        }
    }

    fn bind_active_mission(&mut self, mission: &GatewayMissionSnapshot) {
        self.config.gateway.mission_id = Some(mission.mission_id.clone());
        self.config.gateway.session_key = mission.session_key.clone();
        self.status.active_mission_id = Some(mission.mission_id.clone());
    }
}

fn map_gateway_tool_status(status: GatewayToolStatus) -> ToolStatus {
    match status {
        GatewayToolStatus::Success => ToolStatus::Success,
        GatewayToolStatus::Failed => ToolStatus::Failed,
        GatewayToolStatus::Timeout => ToolStatus::Timeout,
    }
}

fn render_mission_list(missions: &[GatewayMissionSnapshot]) -> String {
    if missions.is_empty() {
        return "No persisted missions found.".to_string();
    }

    let mut lines = vec!["Recent missions:".to_string()];
    for mission in missions {
        lines.push(format!(
            "- {} [{}] session={} attempts={} verifier={} goal={}",
            mission.mission_id,
            mission.status,
            mission.session_key,
            mission.iteration_count,
            mission.latest_verifier.reason_code,
            mission.goal_summary
        ));
    }
    lines.join("\n")
}

fn render_mission_detail(mission: &GatewayMissionSnapshot) -> String {
    let mut lines = vec![
        format!("Mission {}", mission.mission_id),
        format!("status: {}", mission.status),
        format!("session: {}", mission.session_key),
        format!("goal: {}", mission.goal_summary),
        format!(
            "latest verifier: {} ({})",
            mission.latest_verifier.reason_code, mission.latest_verifier.status
        ),
        format!("iterations: {}", mission.iteration_count),
    ];
    if !mission.latest_output_summary.trim().is_empty() {
        lines.push(format!("latest output: {}", mission.latest_output_summary));
    }
    if let Some(completion) = mission.latest_completion.as_ref() {
        lines.push(format!(
            "latest completion: {} - {}",
            completion.status, completion.summary
        ));
        if let Some(next_step) = completion.next_step.as_deref() {
            lines.push(format!("next step: {next_step}"));
        }
    }
    lines.join("\n")
}
