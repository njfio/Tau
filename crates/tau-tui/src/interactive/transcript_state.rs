use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::chat::ChatMessage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct InteractiveTranscriptState {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
}

fn default_schema_version() -> u32 {
    1
}

pub(crate) fn load_interactive_transcript_state(path: &Path) -> Option<InteractiveTranscriptState> {
    let body = match fs::read_to_string(path) {
        Ok(body) => body,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(_) => return None,
    };
    serde_json::from_str(&body).ok()
}

pub(crate) fn save_interactive_transcript_state(
    path: &Path,
    state: &InteractiveTranscriptState,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = serde_json::to_string_pretty(state)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, body)
}
