//! Tool execution panel showing active and recent tool calls.

/// Status of a tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Success,
    Failed,
    Timeout,
}

impl ToolStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ToolStatus::Running => "...",
            ToolStatus::Success => "OK",
            ToolStatus::Failed => "ERR",
            ToolStatus::Timeout => "T/O",
        }
    }

    pub fn accent_name(&self) -> &'static str {
        match self {
            ToolStatus::Running => "running",
            ToolStatus::Success => "ok",
            ToolStatus::Failed => "failed",
            ToolStatus::Timeout => "timed out",
        }
    }
}

/// A single tool execution entry.
#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub status: ToolStatus,
    pub detail: String,
    pub timestamp: String,
}

/// Panel showing tool execution history.
pub struct ToolPanel {
    entries: Vec<ToolEntry>,
    max_entries: usize,
    scroll_offset: usize,
}

impl ToolPanel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 200,
            scroll_offset: 0,
        }
    }

    pub fn add_entry(&mut self, entry: ToolEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub fn entries(&self) -> &[ToolEntry] {
        &self.entries
    }

    pub fn latest_entry(&self) -> Option<&ToolEntry> {
        self.entries.last()
    }

    pub fn latest_running(&self) -> Option<&ToolEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.status == ToolStatus::Running)
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.entries.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    pub fn active_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.status == ToolStatus::Running)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }
}
