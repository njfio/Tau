//! Shared live-memory backend abstraction and default JSONL implementation.
//!
//! This crate intentionally focuses on production memory persistence concerns so
//! contract replay crates can stay deterministic and runtime crates can share a
//! single backend implementation.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

const LIVE_MEMORY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Enumerates supported `LiveMemoryRole` values.
pub enum LiveMemoryRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Public struct `LiveMemoryMessage` used across Tau components.
pub struct LiveMemoryMessage {
    pub role: LiveMemoryRole,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedLiveMemoryEntry {
    schema_version: u32,
    entry_id: String,
    workspace_id: String,
    role: LiveMemoryRole,
    text: String,
    created_unix_ms: u64,
}

/// Trait contract for `LiveMemoryBackend` behavior.
pub trait LiveMemoryBackend: Send + Sync {
    fn state_file(&self, workspace_id: &str) -> PathBuf;

    fn load_messages(&self, workspace_id: &str) -> Result<Vec<LiveMemoryMessage>, String>;

    fn append_messages(
        &self,
        workspace_id: &str,
        messages: &[LiveMemoryMessage],
        max_entries: usize,
    ) -> Result<(), String>;
}

#[derive(Debug, Clone)]
/// Public struct `JsonlLiveMemoryBackend` used across Tau components.
pub struct JsonlLiveMemoryBackend {
    backend_dir: PathBuf,
}

impl JsonlLiveMemoryBackend {
    pub fn from_state_dir(state_dir: &Path) -> Option<Self> {
        if state_dir.as_os_str().is_empty() {
            return None;
        }
        let backend_dir = state_dir.join("live-backend");
        fs::create_dir_all(&backend_dir).ok()?;
        Some(Self { backend_dir })
    }

    fn load_entries(&self, workspace_id: &str) -> Result<Vec<PersistedLiveMemoryEntry>, String> {
        let state_file = self.state_file(workspace_id);
        if !state_file.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(&state_file).map_err(|error| {
            format!(
                "failed to read memory backend state '{}': {error}",
                state_file.display()
            )
        })?;
        let normalized_workspace_id = normalize_workspace_id(workspace_id);
        let mut entries = Vec::new();
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(entry) = serde_json::from_str::<PersistedLiveMemoryEntry>(line) else {
                continue;
            };
            if entry.schema_version != LIVE_MEMORY_SCHEMA_VERSION {
                continue;
            }
            if entry.workspace_id.trim() != normalized_workspace_id {
                continue;
            }
            if entry.text.trim().is_empty() {
                continue;
            }
            entries.push(entry);
        }
        entries.sort_by(|left, right| left.created_unix_ms.cmp(&right.created_unix_ms));
        Ok(entries)
    }
}

impl LiveMemoryBackend for JsonlLiveMemoryBackend {
    fn state_file(&self, workspace_id: &str) -> PathBuf {
        let normalized = normalize_workspace_id(workspace_id);
        self.backend_dir.join(format!("{normalized}.jsonl"))
    }

    fn load_messages(&self, workspace_id: &str) -> Result<Vec<LiveMemoryMessage>, String> {
        let entries = self.load_entries(workspace_id)?;
        Ok(entries
            .into_iter()
            .map(|entry| LiveMemoryMessage {
                role: entry.role,
                text: entry.text,
            })
            .collect())
    }

    fn append_messages(
        &self,
        workspace_id: &str,
        messages: &[LiveMemoryMessage],
        max_entries: usize,
    ) -> Result<(), String> {
        if max_entries == 0 {
            return Ok(());
        }

        let normalized_workspace_id = normalize_workspace_id(workspace_id);
        let state_file = self.state_file(normalized_workspace_id.as_str());
        let mut entries = self
            .load_entries(normalized_workspace_id.as_str())
            .unwrap_or_default();
        let now_unix_ms = current_unix_timestamp_ms();
        let mut appended = 0usize;
        for message in messages {
            let text = collapse_whitespace(message.text.as_str());
            if text.trim().is_empty() {
                continue;
            }
            let created_unix_ms = now_unix_ms.saturating_add(appended as u64);
            let hash_input = format!(
                "{}:{}:{}:{}",
                normalized_workspace_id,
                role_label(message.role),
                created_unix_ms,
                text
            );
            let entry_id = format!("mem_{:016x}", fnv1a_hash(hash_input.as_bytes()));
            entries.push(PersistedLiveMemoryEntry {
                schema_version: LIVE_MEMORY_SCHEMA_VERSION,
                entry_id,
                workspace_id: normalized_workspace_id.clone(),
                role: message.role,
                text,
                created_unix_ms,
            });
            appended = appended.saturating_add(1);
        }
        if appended == 0 {
            return Ok(());
        }

        if entries.len() > max_entries {
            let drop_count = entries.len().saturating_sub(max_entries);
            entries.drain(0..drop_count);
        }

        let mut payload = String::new();
        for entry in entries {
            let line = serde_json::to_string(&entry)
                .map_err(|error| format!("failed to serialize memory backend entry: {error}"))?;
            payload.push_str(line.as_str());
            payload.push('\n');
        }
        fs::write(&state_file, payload).map_err(|error| {
            format!(
                "failed to write memory backend state '{}': {error}",
                state_file.display()
            )
        })?;
        Ok(())
    }
}

pub fn normalize_workspace_id(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "default".to_string();
    }
    let mut normalized = String::with_capacity(trimmed.len());
    for character in trimmed.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push('-');
        }
    }
    let normalized = normalized.trim_matches('-').to_string();
    if normalized.is_empty() {
        "default".to_string()
    } else {
        normalized
    }
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn role_label(role: LiveMemoryRole) -> &'static str {
    match role {
        LiveMemoryRole::User => "user",
        LiveMemoryRole::Assistant => "assistant",
    }
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{JsonlLiveMemoryBackend, LiveMemoryBackend, LiveMemoryMessage, LiveMemoryRole};

    #[test]
    fn unit_normalize_workspace_id_rewrites_unsupported_chars() {
        assert_eq!(
            super::normalize_workspace_id("  Ops / Team  "),
            "ops---team"
        );
        assert_eq!(super::normalize_workspace_id(""), "default");
        assert_eq!(super::normalize_workspace_id("___"), "___");
    }

    #[test]
    fn functional_jsonl_backend_roundtrips_messages() {
        let temp = tempfile::tempdir().expect("tempdir");
        let backend = JsonlLiveMemoryBackend::from_state_dir(temp.path()).expect("backend");
        backend
            .append_messages(
                "workspace-a",
                &[
                    LiveMemoryMessage {
                        role: LiveMemoryRole::User,
                        text: "postgres failover checklist".to_string(),
                    },
                    LiveMemoryMessage {
                        role: LiveMemoryRole::Assistant,
                        text: "promote replica and verify lag".to_string(),
                    },
                ],
                32,
            )
            .expect("append");
        let loaded = backend.load_messages("workspace-a").expect("load");
        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].text.contains("postgres failover"));
        assert!(loaded[1].text.contains("verify lag"));
    }

    #[test]
    fn regression_jsonl_backend_respects_entry_cap() {
        let temp = tempfile::tempdir().expect("tempdir");
        let backend = JsonlLiveMemoryBackend::from_state_dir(temp.path()).expect("backend");
        backend
            .append_messages(
                "workspace-cap",
                &[
                    LiveMemoryMessage {
                        role: LiveMemoryRole::User,
                        text: "entry-one".to_string(),
                    },
                    LiveMemoryMessage {
                        role: LiveMemoryRole::Assistant,
                        text: "entry-two".to_string(),
                    },
                    LiveMemoryMessage {
                        role: LiveMemoryRole::User,
                        text: "entry-three".to_string(),
                    },
                ],
                2,
            )
            .expect("append");
        let loaded = backend.load_messages("workspace-cap").expect("load");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].text, "entry-two");
        assert_eq!(loaded[1].text, "entry-three");
    }

    #[test]
    fn integration_from_state_dir_returns_none_when_unusable() {
        let temp = tempfile::tempdir().expect("tempdir");
        let blocked = temp.path().join("blocked");
        std::fs::write(&blocked, "file").expect("write blocked path");
        let backend = JsonlLiveMemoryBackend::from_state_dir(blocked.as_path());
        assert!(backend.is_none());
    }
}
