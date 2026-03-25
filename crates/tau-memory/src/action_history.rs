//! Persistent action history store for cross-session learning.
//!
//! Records tool executions, LLM requests, replans, and other agent actions
//! to enable learning from past behavior.

use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Types of recorded actions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    ToolExecution,
    LlmRequest,
    Replan,
    BranchSpawn,
    ContextCompaction,
}

/// A single recorded action from an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub session_id: String,
    pub turn: usize,
    pub action_type: ActionType,
    pub tool_name: Option<String>,
    pub input_summary: String,
    pub output_summary: String,
    pub success: bool,
    pub latency_ms: u64,
    pub timestamp_ms: u64,
}

/// Filter criteria for querying action history.
#[derive(Debug, Clone, Default)]
pub struct ActionFilter {
    pub session_id: Option<String>,
    pub action_type: Option<ActionType>,
    pub tool_name: Option<String>,
    pub success: Option<bool>,
    pub max_results: Option<usize>,
}

/// A detected failure pattern from historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub tool_name: String,
    pub common_error: String,
    pub occurrence_count: usize,
}

/// Session feedback recorded at session end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFeedback {
    pub session_id: String,
    pub outcome: SessionOutcome,
    pub tool_effectiveness: HashMap<String, ToolEffectiveness>,
}

/// Outcome of an agent session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutcome {
    Completed,
    Abandoned,
    Failed,
    PartialSuccess,
}

/// Per-tool effectiveness within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEffectiveness {
    pub executions: u64,
    pub successes: u64,
    pub avg_latency_ms: u64,
}

/// Configuration for the action history store.
#[derive(Debug, Clone)]
pub struct ActionHistoryConfig {
    pub store_path: PathBuf,
    pub max_records_per_session: usize,
    pub max_total_records: usize,
}

impl Default for ActionHistoryConfig {
    fn default() -> Self {
        Self {
            store_path: PathBuf::from(".tau/action_history.jsonl"),
            max_records_per_session: 500,
            max_total_records: 10_000,
        }
    }
}

/// In-memory action history store with JSONL persistence.
pub struct ActionHistoryStore {
    config: ActionHistoryConfig,
    records: Vec<ActionRecord>,
}

impl ActionHistoryStore {
    /// Create a new action history store.
    pub fn new(config: ActionHistoryConfig) -> Self {
        Self {
            config,
            records: Vec::new(),
        }
    }

    /// Load an action history store from a JSONL file.
    ///
    /// Reads each line as a JSON-serialized `ActionRecord`, filtering out
    /// records older than `retention_days`. If the file does not exist,
    /// returns an empty store.
    pub fn load(path: &Path, retention_days: u32, config: ActionHistoryConfig) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self {
                config,
                records: Vec::new(),
            });
        }

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let cutoff_ms = now_ms.saturating_sub(retention_days as u64 * 86_400 * 1_000);

        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<ActionRecord>(trimmed) {
                Ok(record) => {
                    if record.timestamp_ms >= cutoff_ms {
                        records.push(record);
                    }
                }
                Err(e) => {
                    tracing::warn!("Skipping malformed action history line: {}", e);
                }
            }
        }

        Ok(Self { config, records })
    }

    /// Save all records to a JSONL file using atomic rename.
    ///
    /// Writes to a `.tmp` file first, then renames to the final path.
    /// Creates parent directories if they don't exist.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_extension("jsonl.tmp");
        {
            let file = std::fs::File::create(&tmp_path)?;
            let mut writer = BufWriter::new(file);
            for record in &self.records {
                let line = serde_json::to_string(record).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                })?;
                writeln!(writer, "{}", line)?;
            }
            writer.flush()?;
        }

        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Append an action record.
    pub fn record(&mut self, action: ActionRecord) {
        self.records.push(action);
        // Enforce max records
        if self.records.len() > self.config.max_total_records {
            let excess = self.records.len() - self.config.max_total_records;
            self.records.drain(..excess);
        }
    }

    /// Query actions by filter criteria.
    pub fn query(&self, filter: &ActionFilter) -> Vec<&ActionRecord> {
        let max = filter.max_results.unwrap_or(usize::MAX);
        self.records
            .iter()
            .filter(|r| {
                if let Some(ref sid) = filter.session_id {
                    if &r.session_id != sid {
                        return false;
                    }
                }
                if let Some(ref at) = filter.action_type {
                    if &r.action_type != at {
                        return false;
                    }
                }
                if let Some(ref tn) = filter.tool_name {
                    if r.tool_name.as_ref() != Some(tn) {
                        return false;
                    }
                }
                if let Some(success) = filter.success {
                    if r.success != success {
                        return false;
                    }
                }
                true
            })
            .take(max)
            .collect()
    }

    /// Get tool success rates across recent records.
    pub fn tool_success_rates(&self, lookback: usize) -> HashMap<String, f64> {
        let recent = &self.records[self.records.len().saturating_sub(lookback)..];
        let mut counts: HashMap<String, (u64, u64)> = HashMap::new(); // (total, success)

        for record in recent {
            if let Some(ref tool) = record.tool_name {
                let entry = counts.entry(tool.clone()).or_default();
                entry.0 += 1;
                if record.success {
                    entry.1 += 1;
                }
            }
        }

        counts
            .into_iter()
            .map(|(name, (total, success))| {
                let rate = if total > 0 {
                    success as f64 / total as f64
                } else {
                    1.0
                };
                (name, rate)
            })
            .collect()
    }

    /// Get common failure patterns.
    pub fn failure_patterns(&self, lookback: usize) -> Vec<FailurePattern> {
        let recent = &self.records[self.records.len().saturating_sub(lookback)..];
        let mut pattern_counts: HashMap<(String, String), usize> = HashMap::new();

        for record in recent {
            if !record.success {
                if let Some(ref tool) = record.tool_name {
                    let key = (tool.clone(), record.output_summary.clone());
                    *pattern_counts.entry(key).or_default() += 1;
                }
            }
        }

        let mut patterns: Vec<FailurePattern> = pattern_counts
            .into_iter()
            .map(|((tool_name, common_error), count)| FailurePattern {
                tool_name,
                common_error,
                occurrence_count: count,
            })
            .collect();

        patterns.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
        patterns
    }

    /// Total number of records stored.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if no records are stored.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn make_record(tool: &str, success: bool) -> ActionRecord {
        ActionRecord {
            session_id: "s1".to_string(),
            turn: 1,
            action_type: ActionType::ToolExecution,
            tool_name: Some(tool.to_string()),
            input_summary: "input".to_string(),
            output_summary: if success {
                "ok".to_string()
            } else {
                "error: failed".to_string()
            },
            success,
            latency_ms: 100,
            timestamp_ms: now_ms(),
        }
    }

    #[test]
    fn record_and_query() {
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", true));
        store.record(make_record("read", false));

        let all = store.query(&ActionFilter::default());
        assert_eq!(all.len(), 2);

        let failures = store.query(&ActionFilter {
            success: Some(false),
            ..Default::default()
        });
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn tool_success_rates() {
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", true));
        store.record(make_record("bash", false));
        store.record(make_record("read", true));

        let rates = store.tool_success_rates(100);
        assert!((rates["bash"] - 0.5).abs() < f64::EPSILON);
        assert!((rates["read"] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn failure_patterns_sorted() {
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", false));
        store.record(make_record("bash", false));
        store.record(make_record("read", false));

        let patterns = store.failure_patterns(100);
        assert!(!patterns.is_empty());
        // bash should have more failures
        assert_eq!(patterns[0].tool_name, "bash");
    }

    #[test]
    fn max_records_enforced() {
        let config = ActionHistoryConfig {
            max_total_records: 5,
            ..Default::default()
        };
        let mut store = ActionHistoryStore::new(config);
        for i in 0..10 {
            store.record(ActionRecord {
                session_id: format!("s{}", i),
                turn: i,
                action_type: ActionType::ToolExecution,
                tool_name: Some("bash".to_string()),
                input_summary: String::new(),
                output_summary: String::new(),
                success: true,
                latency_ms: 0,
                timestamp_ms: i as u64,
            });
        }
        assert_eq!(store.len(), 5);
    }

    #[test]
    fn jsonl_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");

        // Create store with records, save to disk
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", true));
        store.record(make_record("read", false));
        store.save(&path).unwrap();

        // Load from disk and verify records match
        let loaded = ActionHistoryStore::load(&path, 30, ActionHistoryConfig::default()).unwrap();
        assert_eq!(loaded.len(), 2);

        let all = loaded.query(&ActionFilter::default());
        assert_eq!(all[0].tool_name.as_deref(), Some("bash"));
        assert!(all[0].success);
        assert_eq!(all[1].tool_name.as_deref(), Some("read"));
        assert!(!all[1].success);
    }

    #[test]
    fn retention_pruning_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let old_ms = now_ms - (2 * 86_400 * 1_000); // 2 days ago

        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        // Record an old action
        store.record(ActionRecord {
            session_id: "old".to_string(),
            turn: 1,
            action_type: ActionType::ToolExecution,
            tool_name: Some("bash".to_string()),
            input_summary: "old".to_string(),
            output_summary: "ok".to_string(),
            success: true,
            latency_ms: 50,
            timestamp_ms: old_ms,
        });
        // Record a recent action
        store.record(ActionRecord {
            session_id: "new".to_string(),
            turn: 1,
            action_type: ActionType::ToolExecution,
            tool_name: Some("read".to_string()),
            input_summary: "new".to_string(),
            output_summary: "ok".to_string(),
            success: true,
            latency_ms: 50,
            timestamp_ms: now_ms,
        });
        store.save(&path).unwrap();

        // Load with retention_days=1 => old record should be pruned
        let loaded = ActionHistoryStore::load(&path, 1, ActionHistoryConfig::default()).unwrap();
        assert_eq!(loaded.len(), 1);
        let all = loaded.query(&ActionFilter::default());
        assert_eq!(all[0].session_id, "new");
    }

    #[test]
    fn load_missing_file_returns_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.jsonl");

        let loaded = ActionHistoryStore::load(&path, 30, ActionHistoryConfig::default()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep").join("nested").join("history.jsonl");

        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", true));
        store.save(&path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn save_uses_atomic_rename() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        let tmp_path = dir.path().join("history.jsonl.tmp");

        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", true));
        store.save(&path).unwrap();

        // After save completes, .tmp should NOT exist (was renamed)
        assert!(!tmp_path.exists());
        // But the final file should exist
        assert!(path.exists());
    }
}
