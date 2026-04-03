//! Gateway learning helpers built on Tau action history primitives.

use super::*;
use tau_memory::action_history::{
    ActionHistoryConfig, ActionHistoryStore, ActionRecord, ActionType,
};

pub(super) const GATEWAY_ACTION_HISTORY_LOOKBACK: usize = 128;

const GATEWAY_ACTION_HISTORY_FILE: &str = "action-history.jsonl";

#[derive(Debug, Clone)]
pub(super) struct GatewayActionHistoryToolRecord {
    pub(super) session_key: String,
    pub(super) mission_id: String,
    pub(super) turn: usize,
    pub(super) tool_name: String,
    pub(super) input_summary: String,
    pub(super) output_summary: String,
    pub(super) success: bool,
    pub(super) latency_ms: u64,
    pub(super) timestamp_ms: u64,
}

pub(super) fn gateway_action_history_path(state_dir: &Path) -> PathBuf {
    state_dir
        .join("openresponses")
        .join(GATEWAY_ACTION_HISTORY_FILE)
}

pub(super) fn load_gateway_action_history_store(
    state_dir: &Path,
) -> Result<ActionHistoryStore, OpenResponsesApiError> {
    let path = gateway_action_history_path(state_dir);
    let config = ActionHistoryConfig {
        store_path: path.clone(),
        ..ActionHistoryConfig::default()
    };
    ActionHistoryStore::load(&path, 30, config).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to load gateway action history '{}': {error}",
            path.display()
        ))
    })
}

pub(super) fn save_gateway_action_history_store(
    state_dir: &Path,
    store: &ActionHistoryStore,
) -> Result<(), OpenResponsesApiError> {
    let path = gateway_action_history_path(state_dir);
    store.save(&path).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to persist gateway action history '{}': {error}",
            path.display()
        ))
    })
}

pub(super) fn append_gateway_action_history_records(
    store: &mut ActionHistoryStore,
    records: &[GatewayActionHistoryToolRecord],
) {
    if records.is_empty() {
        return;
    }
    for record in records {
        store.record(ActionRecord {
            session_id: record.session_key.clone(),
            turn: record.turn,
            action_type: ActionType::ToolExecution,
            tool_name: Some(record.tool_name.clone()),
            input_summary: format!("mission={} {}", record.mission_id, record.input_summary)
                .trim()
                .to_string(),
            output_summary: record.output_summary.clone(),
            success: record.success,
            latency_ms: record.latency_ms,
            timestamp_ms: record.timestamp_ms,
        });
    }
}

pub(super) fn build_gateway_learning_insight(
    store: &ActionHistoryStore,
    lookback: usize,
) -> LearningInsight {
    let failing_tools = store
        .failure_patterns(lookback)
        .into_iter()
        .map(|pattern| {
            (
                pattern.tool_name,
                pattern.common_error,
                pattern.occurrence_count,
            )
        })
        .collect::<Vec<_>>();
    let tool_success_rates = store
        .tool_success_rates(lookback)
        .into_iter()
        .collect::<Vec<_>>();
    LearningInsight {
        failing_tools,
        tool_success_rates,
    }
}

pub(super) fn render_gateway_learning_bulletin(
    store: &ActionHistoryStore,
    lookback: usize,
) -> String {
    format_learning_bulletin(&build_gateway_learning_insight(store, lookback))
}

#[cfg(test)]
fn render_gateway_learning_bulletin_from_disk(
    state_dir: &Path,
    lookback: usize,
) -> Result<String, OpenResponsesApiError> {
    let store = load_gateway_action_history_store(state_dir)?;
    Ok(render_gateway_learning_bulletin(&store, lookback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn unit_gateway_action_history_path_uses_openresponses_root() {
        let path = gateway_action_history_path(Path::new("/tmp/tau-gateway"));
        assert!(path.ends_with("openresponses/action-history.jsonl"));
    }

    #[test]
    fn unit_append_gateway_action_history_records_updates_store_without_disk_io() {
        let temp = tempdir().expect("tempdir");
        let path = gateway_action_history_path(temp.path());
        let mut store = ActionHistoryStore::new(ActionHistoryConfig {
            store_path: path.clone(),
            ..ActionHistoryConfig::default()
        });
        append_gateway_action_history_records(
            &mut store,
            &[GatewayActionHistoryToolRecord {
                session_key: "session-a".to_string(),
                mission_id: "mission-a".to_string(),
                turn: 2,
                tool_name: "bash".to_string(),
                input_summary: "pwd".to_string(),
                output_summary: "ok".to_string(),
                success: true,
                latency_ms: 12,
                timestamp_ms: 44,
            }],
        );
        assert_eq!(store.len(), 1);
        assert!(
            !path.exists(),
            "in-memory append helper must not perform disk I/O by itself"
        );
    }

    #[test]
    fn unit_render_gateway_learning_bulletin_from_store_matches_disk_render() {
        let temp = tempdir().expect("tempdir");
        let now = current_unix_timestamp_ms();
        let mut store = ActionHistoryStore::new(ActionHistoryConfig {
            store_path: gateway_action_history_path(temp.path()),
            ..ActionHistoryConfig::default()
        });
        append_gateway_action_history_records(
            &mut store,
            &[
                GatewayActionHistoryToolRecord {
                    session_key: "session-a".to_string(),
                    mission_id: "mission-a".to_string(),
                    turn: 1,
                    tool_name: "bash".to_string(),
                    input_summary: "cargo test".to_string(),
                    output_summary: "timeout".to_string(),
                    success: false,
                    latency_ms: 90,
                    timestamp_ms: now.saturating_sub(2),
                },
                GatewayActionHistoryToolRecord {
                    session_key: "session-a".to_string(),
                    mission_id: "mission-a".to_string(),
                    turn: 2,
                    tool_name: "read".to_string(),
                    input_summary: "src/lib.rs".to_string(),
                    output_summary: "ok".to_string(),
                    success: true,
                    latency_ms: 5,
                    timestamp_ms: now.saturating_sub(1),
                },
            ],
        );
        let in_memory = render_gateway_learning_bulletin(&store, GATEWAY_ACTION_HISTORY_LOOKBACK);
        save_gateway_action_history_store(temp.path(), &store).expect("save action history");
        let disk_backed = render_gateway_learning_bulletin_from_disk(
            temp.path(),
            GATEWAY_ACTION_HISTORY_LOOKBACK,
        )
        .expect("render disk-backed bulletin");
        assert_eq!(in_memory, disk_backed);
    }
}
