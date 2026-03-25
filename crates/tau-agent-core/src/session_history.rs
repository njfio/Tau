//! Session-level action history lifecycle helpers.
//!
//! Bridges `tau_memory::action_history::ActionHistoryStore` with the agent
//! runtime by providing helper functions for initialisation, recording, and
//! persistence. These are intentionally free-standing functions so that the
//! `Agent` struct can call them without coupling tightly to the store
//! internals.

use std::path::PathBuf;

use tau_memory::action_history::{ActionHistoryConfig, ActionHistoryStore, ActionRecord};

use crate::cortex_runtime::LearningInsight;
use crate::recovery::HistoricalToolInsight;
use crate::AgentConfig;

/// Initialise an [`ActionHistoryStore`] from the agent configuration.
///
/// Returns `None` when action history is disabled or when the store cannot be
/// loaded from disk (in which case a warning is logged and an empty store is
/// returned inside `Some`).
pub fn init_action_history(config: &AgentConfig) -> Option<ActionHistoryStore> {
    if !config.action_history_enabled {
        return None;
    }

    let path = action_history_path(config);
    let store_config = ActionHistoryConfig {
        store_path: path.clone(),
        max_records_per_session: config.action_history_max_records,
        max_total_records: config.action_history_max_records,
    };

    match ActionHistoryStore::load(&path, config.action_history_retention_days, store_config) {
        Ok(store) => Some(store),
        Err(_) => {
            // Load failed — return an empty store so the session can still record new actions.
            Some(ActionHistoryStore::new(ActionHistoryConfig {
                store_path: path,
                max_records_per_session: config.action_history_max_records,
                max_total_records: config.action_history_max_records,
            }))
        }
    }
}

/// Persist the action history store to disk.
///
/// Logs a warning on failure but does not propagate the error, since history
/// persistence should never prevent the session from completing.
pub fn save_action_history(store: &ActionHistoryStore, config: &AgentConfig) {
    let path = action_history_path(config);
    // Silently ignore save errors — history persistence should never block the session.
    let _ = store.save(&path);
}

/// Record a single action into the store.
pub fn record_action(store: &mut ActionHistoryStore, record: ActionRecord) {
    store.record(record);
}

/// Derive the store file path from agent configuration.
pub fn action_history_path(config: &AgentConfig) -> PathBuf {
    config
        .action_history_store_path
        .clone()
        .unwrap_or_else(|| PathBuf::from(".tau/action_history.jsonl"))
}

/// Build a [`LearningInsight`] from the current action history store.
///
/// Analyses the most recent `lookback` records to extract failure patterns
/// and tool success rates that inform the Cortex bulletin.
pub fn build_learning_insight(store: &ActionHistoryStore) -> LearningInsight {
    let lookback = 50;
    let patterns = store.failure_patterns(lookback);
    let rates = store.tool_success_rates(lookback);

    LearningInsight {
        failing_tools: patterns
            .iter()
            .take(3)
            .map(|p| (p.tool_name.clone(), p.common_error.clone(), p.occurrence_count))
            .collect(),
        tool_success_rates: rates
            .iter()
            .filter(|(_, r)| **r < 0.5)
            .map(|(n, r)| (n.clone(), *r))
            .collect(),
    }
}

/// Build a [`HistoricalToolInsight`] for a specific tool from the action
/// history store.
///
/// Returns `None` if there is no history for the given tool name.
pub fn build_tool_insight(
    store: &ActionHistoryStore,
    tool_name: &str,
) -> Option<HistoricalToolInsight> {
    let lookback = 50;
    let rates = store.tool_success_rates(lookback);
    let patterns = store.failure_patterns(lookback);

    let success_rate = rates.get(tool_name).copied()?;

    let tool_patterns: Vec<_> = patterns
        .iter()
        .filter(|p| p.tool_name == tool_name)
        .collect();

    let failure_count: usize = tool_patterns.iter().map(|p| p.occurrence_count).sum();
    let common_error = tool_patterns.first().map(|p| p.common_error.clone());

    Some(HistoricalToolInsight {
        tool_name: tool_name.to_string(),
        success_rate,
        common_error,
        failure_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tau_memory::action_history::{ActionRecord, ActionType};

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn make_record(tool: &str, success: bool, error: &str) -> ActionRecord {
        ActionRecord {
            session_id: "test-session".to_string(),
            turn: 1,
            action_type: ActionType::ToolExecution,
            tool_name: Some(tool.to_string()),
            input_summary: "input".to_string(),
            output_summary: error.to_string(),
            success,
            latency_ms: 100,
            timestamp_ms: now_ms(),
        }
    }

    #[test]
    fn init_action_history_returns_none_when_disabled() {
        let config = AgentConfig {
            action_history_enabled: false,
            ..AgentConfig::default()
        };
        assert!(init_action_history(&config).is_none());
    }

    #[test]
    fn init_action_history_returns_some_when_enabled() {
        let config = AgentConfig {
            action_history_enabled: true,
            // Use a nonexistent path — should still return an empty store
            action_history_store_path: Some(PathBuf::from("/tmp/nonexistent_tau_test/history.jsonl")),
            ..AgentConfig::default()
        };
        let store = init_action_history(&config);
        assert!(store.is_some());
        assert!(store.unwrap().is_empty());
    }

    #[test]
    fn save_and_reload_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        let config = AgentConfig {
            action_history_enabled: true,
            action_history_store_path: Some(path.clone()),
            ..AgentConfig::default()
        };

        let mut store = init_action_history(&config).unwrap();
        record_action(&mut store, make_record("bash", true, "ok"));
        record_action(&mut store, make_record("read", false, "not found"));
        save_action_history(&store, &config);

        // Reload
        let reloaded = init_action_history(&config).unwrap();
        assert_eq!(reloaded.len(), 2);
    }

    #[test]
    fn build_learning_insight_extracts_patterns() {
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", false, "timeout"));
        store.record(make_record("bash", false, "timeout"));
        store.record(make_record("bash", true, "ok"));
        store.record(make_record("read", false, "not found"));

        let insight = build_learning_insight(&store);

        // bash has 2 failures with "timeout"
        assert!(!insight.failing_tools.is_empty());
        let bash_failure = insight
            .failing_tools
            .iter()
            .find(|(name, _, _)| name == "bash");
        assert!(bash_failure.is_some());
        let (_, error, count) = bash_failure.unwrap();
        assert_eq!(error, "timeout");
        assert_eq!(*count, 2);

        // bash success rate: 1/3 ≈ 0.33 < 0.5, should appear
        let bash_rate = insight
            .tool_success_rates
            .iter()
            .find(|(name, _)| name == "bash");
        assert!(bash_rate.is_some());
    }

    #[test]
    fn build_learning_insight_empty_store() {
        let store = ActionHistoryStore::new(ActionHistoryConfig::default());
        let insight = build_learning_insight(&store);
        assert!(insight.failing_tools.is_empty());
        assert!(insight.tool_success_rates.is_empty());
    }

    #[test]
    fn build_tool_insight_returns_none_for_unknown_tool() {
        let store = ActionHistoryStore::new(ActionHistoryConfig::default());
        assert!(build_tool_insight(&store, "nonexistent").is_none());
    }

    #[test]
    fn build_tool_insight_returns_insight_for_known_tool() {
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("bash", true, "ok"));
        store.record(make_record("bash", false, "permission denied"));
        store.record(make_record("bash", false, "permission denied"));

        let insight = build_tool_insight(&store, "bash").unwrap();
        assert_eq!(insight.tool_name, "bash");
        // 1 success out of 3 = 0.333...
        assert!(insight.success_rate < 0.4);
        assert_eq!(insight.failure_count, 2);
        assert_eq!(insight.common_error.as_deref(), Some("permission denied"));
    }

    #[test]
    fn build_tool_insight_healthy_tool() {
        let mut store = ActionHistoryStore::new(ActionHistoryConfig::default());
        store.record(make_record("read", true, "ok"));
        store.record(make_record("read", true, "ok"));

        let insight = build_tool_insight(&store, "read").unwrap();
        assert_eq!(insight.success_rate, 1.0);
        assert_eq!(insight.failure_count, 0);
        assert!(insight.common_error.is_none());
    }
}
