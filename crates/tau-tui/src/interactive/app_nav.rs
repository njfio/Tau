use super::{App, FocusPanel, InputMode};

impl App {
    pub(super) fn enter_insert_mode(&mut self, move_end: bool) {
        self.input_mode = InputMode::Insert;
        self.focus = FocusPanel::Input;
        if move_end {
            self.input.move_end();
        }
    }

    pub(super) fn insert_new_line(&mut self) {
        self.input_mode = InputMode::Insert;
        self.focus = FocusPanel::Input;
        self.input.new_line();
    }

    pub(super) fn scroll_chat_down(&mut self, lines: usize) {
        if self.focus == FocusPanel::Chat {
            self.chat.scroll_down(lines);
        }
    }

    pub(super) fn scroll_chat_up(&mut self, lines: usize) {
        if self.focus == FocusPanel::Chat {
            self.chat.scroll_up(lines);
        }
    }

    pub(super) fn scroll_chat_to_bottom(&mut self) {
        if self.focus == FocusPanel::Chat {
            self.chat.scroll_to_bottom();
        }
    }

    pub(super) fn scroll_chat_to_top(&mut self) {
        if self.focus == FocusPanel::Chat {
            self.chat.scroll_to_top();
        }
    }
}
