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
    state_dir: &Path,
    records: &[GatewayActionHistoryToolRecord],
) -> Result<(), OpenResponsesApiError> {
    if records.is_empty() {
        return Ok(());
    }
    let mut store = load_gateway_action_history_store(state_dir)?;
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
    save_gateway_action_history_store(state_dir, &store)
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
    state_dir: &Path,
    lookback: usize,
) -> Result<String, OpenResponsesApiError> {
    let store = load_gateway_action_history_store(state_dir)?;
    Ok(format_learning_bulletin(&build_gateway_learning_insight(
        &store, lookback,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_gateway_action_history_path_uses_openresponses_root() {
        let path = gateway_action_history_path(Path::new("/tmp/tau-gateway"));
        assert!(path.ends_with("openresponses/action-history.jsonl"));
    }
}
