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
    max_messages: usize,
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            max_messages: 10_000,
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn replace_last_content(&mut self, role: MessageRole, content: String) -> bool {
        let Some(message) = self.messages.last_mut() else {
            return false;
        };
        if message.role != role {
            return false;
        }
        message.content = content;
        true
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.messages.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn latest_content_by_role(&self, role: MessageRole) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|msg| msg.role == role)
            .map(|msg| msg.content.as_str())
    }

    pub fn count_by_role(&self, role: MessageRole) -> usize {
        self.messages.iter().filter(|msg| msg.role == role).count()
    }
}
