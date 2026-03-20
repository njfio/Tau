//! Multi-line input editor with cursor tracking.

/// Multi-line input editor.
pub struct InputEditor {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
}

impl Default for InputEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEditor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        let line = &mut self.lines[self.cursor_line];
        let byte_idx = char_to_byte(line, self.cursor_col);
        line.insert(byte_idx, ch);
        self.cursor_col += 1;
    }

    pub fn new_line(&mut self) {
        let line = &mut self.lines[self.cursor_line];
        let byte_idx = char_to_byte(line, self.cursor_col);
        let tail = line.split_off(byte_idx);
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.lines.insert(self.cursor_line, tail);
    }

    pub fn delete_backward(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_line];
            let byte_idx = char_to_byte(line, self.cursor_col);
            let prev_byte = char_to_byte(line, self.cursor_col - 1);
            line.drain(prev_byte..byte_idx);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            let removed = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].chars().count();
            self.lines[self.cursor_line].push_str(&removed);
        }
    }

    pub fn delete_forward(&mut self) {
        let line_chars = self.lines[self.cursor_line].chars().count();
        if self.cursor_col < line_chars {
            let line = &mut self.lines[self.cursor_line];
            let byte_idx = char_to_byte(line, self.cursor_col);
            let next_byte = char_to_byte(line, self.cursor_col + 1);
            line.drain(byte_idx..next_byte);
        } else if self.cursor_line + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let line_chars = self.lines[self.cursor_line].chars().count();
        if self.cursor_col < line_chars {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let line_chars = self.lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(line_chars);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            let line_chars = self.lines[self.cursor_line].chars().count();
            self.cursor_col = self.cursor_col.min(line_chars);
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_line].chars().count();
    }
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
