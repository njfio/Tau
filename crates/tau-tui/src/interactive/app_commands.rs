use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::status::AgentStateDisplay;
use super::focus::{next_insert_focus, next_normal_focus};
use super::{App, ApprovalRequest, DetailSection, FocusPanel, InputMode};

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.handle_global_shortcut(key) {
            return;
        }
        if self.focus == FocusPanel::CommandPalette {
            self.handle_command_palette_key(key);
            return;
        }
        if self.show_help {
            self.show_help = false;
            return;
        }
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Insert => self.handle_insert_mode(key),
        }
    }

    fn handle_global_shortcut(&mut self, key: KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.should_quit = true,
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => self.chat.clear(),
            (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
                self.show_tool_panel = !self.show_tool_panel;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => self.toggle_command_palette(),
            _ => return false,
        }
        true
    }

    fn toggle_command_palette(&mut self) {
        if self.focus == FocusPanel::CommandPalette {
            self.focus = FocusPanel::Input;
            return;
        }
        self.focus = FocusPanel::CommandPalette;
        self.command_input.clear();
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        if self.handle_scroll_or_detail_key(key) {
            return;
        }
        match key.code {
            KeyCode::Char('i') => self.enter_insert_mode(false),
            KeyCode::Char('a') => self.enter_insert_mode(true),
            KeyCode::Char('o') => self.insert_new_line(),
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('y') if self.approval_request.is_some() => self.resolve_approval(true),
            KeyCode::Char('n') if self.approval_request.is_some() => self.resolve_approval(false),
            KeyCode::Char('r') if self.status.agent_state == AgentStateDisplay::Error => {
                self.retry_last_prompt()
            }
            KeyCode::Tab => self.focus = next_normal_focus(self.focus, self.show_tool_panel),
            KeyCode::Char('1') => self.focus = FocusPanel::Chat,
            KeyCode::Char('2') => self.focus = FocusPanel::Input,
            KeyCode::Char('3') if self.show_tool_panel => self.focus = FocusPanel::Tools,
            _ => {}
        }
    }

    fn handle_scroll_or_detail_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.scroll_chat_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_chat_up(1),
            KeyCode::Char('G') => self.scroll_chat_to_bottom(),
            KeyCode::Char('g') => self.scroll_chat_to_top(),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_chat_down(10)
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.scroll_chat_up(10)
            }
            KeyCode::Char('[') if self.focus == FocusPanel::Tools => {
                self.cycle_detail_section_backward()
            }
            KeyCode::Char(']') if self.focus == FocusPanel::Tools => {
                self.cycle_detail_section_forward()
            }
            _ => return false,
        }
        true
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => self.input_mode = InputMode::Normal,
            (KeyModifiers::NONE, KeyCode::Char('y')) if self.approval_request.is_some() => {
                self.resolve_approval(true)
            }
            (KeyModifiers::NONE, KeyCode::Char('n')) if self.approval_request.is_some() => {
                self.resolve_approval(false)
            }
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Enter) => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.input.new_line();
                } else {
                    self.submit_input();
                }
            }
            (KeyModifiers::ALT, KeyCode::Enter) => self.input.new_line(),
            (_, KeyCode::Char(c)) => self.input.insert_char(c),
            (_, KeyCode::Backspace) => self.input.delete_backward(),
            (_, KeyCode::Delete) => self.input.delete_forward(),
            (_, KeyCode::Left) => self.input.move_left(),
            (_, KeyCode::Right) => self.input.move_right(),
            (_, KeyCode::Up) => self.input.move_up(),
            (_, KeyCode::Down) => self.input.move_down(),
            (_, KeyCode::Home) => self.input.move_home(),
            (_, KeyCode::End) => self.input.move_end(),
            (_, KeyCode::Tab) => self.focus = next_insert_focus(self.focus, self.show_tool_panel),
            _ => {}
        }
    }

    fn handle_command_palette_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.focus = FocusPanel::Input;
                self.command_input.clear();
            }
            KeyCode::Enter => {
                let cmd = self.command_input.clone();
                self.focus = FocusPanel::Input;
                self.command_input.clear();
                self.execute_command(&cmd);
            }
            KeyCode::Char(c) => self.command_input.push(c),
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            _ => {}
        }
    }

    pub(super) fn execute_command(&mut self, cmd: &str) {
        match cmd.trim() {
            "quit" | "q" => self.should_quit = true,
            "clear" => self.chat.clear(),
            "help" => self.show_help = true,
            "details" => self.show_tool_panel = !self.show_tool_panel,
            "tools" => self.show_detail_section(DetailSection::Tools),
            "memory" => self.show_detail_section(DetailSection::Memory),
            "cortex" => self.show_detail_section(DetailSection::Cortex),
            "sessions" => self.show_detail_section(DetailSection::Sessions),
            "approval-needed" => {
                self.approval_request = Some(ApprovalRequest {
                    summary: "Tool access needs confirmation before continuing.".to_string(),
                });
            }
            "approve" => self.resolve_approval(true),
            "reject" => self.resolve_approval(false),
            "interrupt" => {
                self.status.agent_state = AgentStateDisplay::Idle;
                self.push_system_note("Interrupt requested.");
            }
            "retry" => self.retry_last_prompt(),
            _ => self.push_system_note(&format!("Unknown command: {cmd}")),
        }
    }
    fn resolve_approval(&mut self, approved: bool) {
        let Some(request) = self.approval_request.take() else {
            return;
        };
        let outcome = if approved { "approved" } else { "rejected" };
        self.push_system_note(&format!("Approval {outcome}: {}", request.summary));
    }
}
