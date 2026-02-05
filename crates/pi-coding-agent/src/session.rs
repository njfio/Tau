use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use pi_ai::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub message: Message,
}

#[derive(Debug)]
pub struct SessionStore {
    path: PathBuf,
    entries: Vec<SessionEntry>,
    next_id: u64,
}

impl SessionStore {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut entries = Vec::new();

        if path.exists() {
            let file = fs::File::open(&path)
                .with_context(|| format!("failed to open session file {}", path.display()))?;
            let reader = BufReader::new(file);

            for (index, line) in reader.lines().enumerate() {
                let line = line.with_context(|| {
                    format!("failed to read line {} from {}", index + 1, path.display())
                })?;

                if line.trim().is_empty() {
                    continue;
                }

                let entry: SessionEntry = serde_json::from_str(&line).with_context(|| {
                    format!(
                        "failed to parse session line {} in {}",
                        index + 1,
                        path.display()
                    )
                })?;
                entries.push(entry);
            }
        }

        let next_id = entries.iter().map(|entry| entry.id).max().unwrap_or(0) + 1;
        Ok(Self {
            path,
            entries,
            next_id,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn entries(&self) -> &[SessionEntry] {
        &self.entries
    }

    pub fn head_id(&self) -> Option<u64> {
        self.entries.last().map(|entry| entry.id)
    }

    pub fn contains(&self, id: u64) -> bool {
        self.entries.iter().any(|entry| entry.id == id)
    }

    pub fn ensure_initialized(&mut self, system_prompt: &str) -> Result<Option<u64>> {
        if !self.entries.is_empty() {
            return Ok(self.head_id());
        }

        if system_prompt.trim().is_empty() {
            return Ok(None);
        }

        let system_message = Message::system(system_prompt);
        self.append_messages(None, &[system_message])
    }

    pub fn append_messages(
        &mut self,
        mut parent_id: Option<u64>,
        messages: &[Message],
    ) -> Result<Option<u64>> {
        if messages.is_empty() {
            return Ok(parent_id);
        }

        if let Some(parent) = parent_id {
            if !self.contains(parent) {
                bail!("parent id {parent} does not exist in session");
            }
        }

        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create session directory {}", parent.display())
                })?;
            }
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("failed to open {} for appending", self.path.display()))?;

        for message in messages {
            let entry = SessionEntry {
                id: self.next_id,
                parent_id,
                message: message.clone(),
            };
            self.next_id += 1;

            let line = serde_json::to_string(&entry)?;
            writeln!(file, "{line}")
                .with_context(|| format!("failed to write session file {}", self.path.display()))?;

            parent_id = Some(entry.id);
            self.entries.push(entry);
        }

        Ok(parent_id)
    }

    pub fn lineage_messages(&self, head_id: Option<u64>) -> Result<Vec<Message>> {
        let Some(mut current_id) = head_id else {
            return Ok(Vec::new());
        };

        let mut ids = Vec::new();
        let mut visited = HashSet::new();

        loop {
            if !visited.insert(current_id) {
                bail!("detected a cycle while resolving session lineage at id {current_id}");
            }

            let entry = self
                .entries
                .iter()
                .find(|entry| entry.id == current_id)
                .ok_or_else(|| anyhow!("unknown session id {current_id}"))?;

            ids.push(entry.id);
            match entry.parent_id {
                Some(parent) => current_id = parent,
                None => break,
            }
        }

        ids.reverse();

        let messages = ids
            .into_iter()
            .map(|id| {
                self.entries
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| entry.message.clone())
                    .ok_or_else(|| anyhow!("missing message for id {id}"))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(messages)
    }

    pub fn branch_tips(&self) -> Vec<&SessionEntry> {
        let mut parent_ids = HashSet::new();
        for entry in &self.entries {
            if let Some(parent_id) = entry.parent_id {
                parent_ids.insert(parent_id);
            }
        }

        let mut tips = self
            .entries
            .iter()
            .filter(|entry| !parent_ids.contains(&entry.id))
            .collect::<Vec<_>>();
        tips.sort_by_key(|entry| entry.id);
        tips
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{SessionEntry, SessionStore};

    #[test]
    fn appends_and_restores_lineage() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");

        let mut store = SessionStore::load(&path).expect("load");
        let head = store
            .append_messages(None, &[pi_ai::Message::system("sys")])
            .expect("append");
        let head = store
            .append_messages(
                head,
                &[
                    pi_ai::Message::user("hello"),
                    pi_ai::Message::assistant_text("hi"),
                ],
            )
            .expect("append");

        let lineage = store.lineage_messages(head).expect("lineage");
        assert_eq!(lineage.len(), 3);
        assert_eq!(lineage[0].text_content(), "sys");

        let reloaded = SessionStore::load(&path).expect("reload");
        assert_eq!(reloaded.entries().len(), 3);
    }

    #[test]
    fn supports_branching_from_older_id() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");

        let mut store = SessionStore::load(&path).expect("load");
        let head = store
            .append_messages(None, &[pi_ai::Message::system("sys")])
            .expect("append");
        let head = store
            .append_messages(
                head,
                &[
                    pi_ai::Message::user("q1"),
                    pi_ai::Message::assistant_text("a1"),
                    pi_ai::Message::user("q2"),
                    pi_ai::Message::assistant_text("a2"),
                ],
            )
            .expect("append");

        let branch_from = Some(head.expect("head") - 2);
        let branch_head = store
            .append_messages(
                branch_from,
                &[
                    pi_ai::Message::user("q2b"),
                    pi_ai::Message::assistant_text("a2b"),
                ],
            )
            .expect("append");

        let lineage = store.lineage_messages(branch_head).expect("lineage");
        let texts = lineage
            .iter()
            .map(|message| message.text_content())
            .collect::<Vec<_>>();
        assert_eq!(texts, vec!["sys", "q1", "a1", "q2b", "a2b"]);
        assert_eq!(store.branch_tips().len(), 2);
    }

    #[test]
    fn append_rejects_unknown_parent_id() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");

        let mut store = SessionStore::load(&path).expect("load");
        let error = store
            .append_messages(Some(42), &[pi_ai::Message::user("hello")])
            .expect_err("must fail for unknown parent");

        assert!(error
            .to_string()
            .contains("parent id 42 does not exist in session"));
    }

    #[test]
    fn detects_cycles_in_session_lineage() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("session.jsonl");

        let entries = vec![
            SessionEntry {
                id: 1,
                parent_id: Some(2),
                message: pi_ai::Message::system("sys"),
            },
            SessionEntry {
                id: 2,
                parent_id: Some(1),
                message: pi_ai::Message::user("hello"),
            },
        ];
        let raw = entries
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()
            .expect("serialize entries")
            .join("\n");
        fs::write(&path, format!("{raw}\n")).expect("write session file");

        let store = SessionStore::load(&path).expect("load");
        let error = store
            .lineage_messages(Some(1))
            .expect_err("lineage should fail for cycle");
        assert!(error.to_string().contains("detected a cycle"));
    }
}
