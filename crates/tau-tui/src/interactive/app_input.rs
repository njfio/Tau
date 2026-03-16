use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, FocusPanel, InputMode};

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
                self.show_tool_panel = !self.show_tool_panel
            }
            (KeyModifiers::CONTROL, KeyCode::Char('p')) => self.toggle_command_palette(),
            _ => return false,
        }
        true
    }

    fn toggle_command_palette(&mut self) {
        self.command_input.clear();
        self.focus = if self.focus == FocusPanel::CommandPalette {
            FocusPanel::Input
        } else {
            FocusPanel::CommandPalette
        };
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        if self.handle_mode_switch(key.code) || self.handle_chat_scroll(key) {
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::Char('1') => self.focus = FocusPanel::Chat,
            KeyCode::Char('2') => self.focus = FocusPanel::Input,
            KeyCode::Char('3') if self.show_tool_panel => self.focus = FocusPanel::Tools,
            _ => {}
        }
    }

    fn handle_mode_switch(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('i') => self.enter_input_mode(false, false),
            KeyCode::Char('a') => self.enter_input_mode(true, false),
            KeyCode::Char('o') => self.enter_input_mode(false, true),
            _ => return false,
        }
        true
    }

    fn enter_input_mode(&mut self, move_end: bool, new_line: bool) {
        self.input_mode = InputMode::Insert;
        self.focus = FocusPanel::Input;
        if move_end {
            self.input.move_end();
        }
        if new_line {
            self.input.new_line();
        }
    }

    fn handle_chat_scroll(&mut self, key: KeyEvent) -> bool {
        if self.focus != FocusPanel::Chat {
            return false;
        }
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('j') | KeyCode::Down) => self.chat.scroll_down(1),
            (_, KeyCode::Char('k') | KeyCode::Up) => self.chat.scroll_up(1),
            (_, KeyCode::Char('G')) => self.chat.scroll_to_bottom(),
            (_, KeyCode::Char('g')) => self.chat.scroll_to_top(),
            (mods, KeyCode::Char('d')) if mods.contains(KeyModifiers::CONTROL) => {
                self.chat.scroll_down(10)
            }
            (mods, KeyCode::Char('u')) if mods.contains(KeyModifiers::CONTROL) => {
                self.chat.scroll_up(10)
            }
            _ => return false,
        }
        true
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => self.input_mode = InputMode::Normal,
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Enter) => {
                self.handle_enter(key.modifiers)
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
            (_, KeyCode::Tab) => self.cycle_focus(),
            _ => {}
        }
    }

    fn handle_enter(&mut self, modifiers: KeyModifiers) {
        if modifiers.contains(KeyModifiers::SHIFT) {
            self.input.new_line();
        } else {
            self.submit_input();
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Chat => FocusPanel::Input,
            FocusPanel::Input => self.next_focus_from_input(),
            FocusPanel::Tools => FocusPanel::Chat,
            FocusPanel::CommandPalette => FocusPanel::Input,
        };
    }

    fn next_focus_from_input(&self) -> FocusPanel {
        if self.show_tool_panel {
            FocusPanel::Tools
        } else {
            FocusPanel::Chat
        }
    }

    fn handle_command_palette_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.toggle_command_palette(),
            KeyCode::Enter => self.submit_command_palette(),
            KeyCode::Char(c) => self.command_input.push(c),
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            _ => {}
        }
    }

    fn submit_command_palette(&mut self) {
        let command = self.command_input.clone();
        self.command_input.clear();
        self.focus = FocusPanel::Input;
        self.execute_command(&command);
    }
}
