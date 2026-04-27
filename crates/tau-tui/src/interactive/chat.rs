//! Scrollable chat panel with message history.

/// Role of a chat message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl MessageRole {
    pub fn label(&self) -> &'static str {
        match self {
            MessageRole::User => "You",
            MessageRole::Assistant => "Tau",
            MessageRole::System => "System",
            MessageRole::Tool => "Tool",
        }
    }
}

/// A single chat message.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
}

/// Scrollable chat panel holding the conversation history.
pub struct ChatPanel {
    messages: Vec<ChatMessage>,
    scroll_offset: usize,
    max_scroll: usize,
    follow_mode: bool,
    max_messages: usize,
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            max_scroll: 0,
            follow_mode: true,
            max_messages: 10_000,
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) -> usize {
        self.messages.push(msg);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        let index = self.messages.len().saturating_sub(1);
        if self.follow_mode {
            self.scroll_offset = self.max_scroll;
        }
        index
    }

    pub fn append_to_message(&mut self, index: usize, content: &str) -> bool {
        let Some(message) = self.messages.get_mut(index) else {
            return false;
        };
        message.content.push_str(content);
        true
    }

    pub fn set_message_content(&mut self, index: usize, content: String) -> bool {
        let Some(message) = self.messages.get_mut(index) else {
            return false;
        };
        message.content = content;
        true
    }

    pub fn message_content(&self, index: usize) -> Option<&str> {
        self.messages
            .get(index)
            .map(|message| message.content.as_str())
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn latest_message_index(&self, role: MessageRole) -> Option<usize> {
        self.messages
            .iter()
            .rposition(|message| message.role == role)
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn follow_mode(&self) -> bool {
        self.follow_mode
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        self.follow_mode = false;
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = (self.scroll_offset + n).min(self.max_scroll);
        if self.scroll_offset >= self.max_scroll {
            self.follow_mode = true;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll;
        self.follow_mode = true;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.follow_mode = false;
    }

    pub fn set_max_scroll(&mut self, max: usize) {
        self.max_scroll = max;
        if self.follow_mode {
            self.scroll_offset = max;
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
        self.max_scroll = 0;
        self.follow_mode = true;
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn last_assistant_content(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, MessageRole::Assistant))
            .map(|m| m.content.as_str())
    }

    pub fn transcript_text(&self) -> String {
        self.messages
            .iter()
            .map(|m| format!("[{}] {}: {}", m.timestamp, m.role.label(), m.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
