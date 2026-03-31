//! Gateway-local mission supervisor persistence for the first Tau Ralph loop slice.

use super::*;

const GATEWAY_MISSION_SCHEMA_VERSION: u32 = 1;
const GATEWAY_MISSION_SUMMARY_MAX_CHARS: usize = 240;
const GATEWAY_MISSION_MAX_ITERATIONS: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum GatewayMissionStatus {
    Running,
    Completed,
    Checkpointed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct GatewayMissionIterationRecord {
    pub(super) attempt: usize,
    pub(super) prompt_summary: String,
    pub(super) assistant_summary: String,
    pub(super) tool_execution_count: usize,
    pub(super) verifier: GatewayMissionVerifierBundle,
    #[serde(default)]
    pub(super) completion: Option<GatewayMissionCompletionSignalRecord>,
    pub(super) started_unix_ms: u64,
    pub(super) finished_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct GatewayMissionState {
    pub(super) schema_version: u32,
    pub(super) mission_id: String,
    pub(super) session_key: String,
    pub(super) response_id: String,
    pub(super) goal_summary: String,
    pub(super) latest_output_summary: String,
    pub(super) status: GatewayMissionStatus,
    pub(super) created_unix_ms: u64,
    pub(super) updated_unix_ms: u64,
    pub(super) iteration_count: usize,
    pub(super) latest_verifier: GatewayMissionVerifierRecord,
    #[serde(default)]
    pub(super) latest_completion: Option<GatewayMissionCompletionSignalRecord>,
    pub(super) iterations: Vec<GatewayMissionIterationRecord>,
}

impl GatewayMissionState {
    pub(super) fn load_or_create(
        path: &Path,
        mission_id: &str,
        session_key: &str,
        response_id: &str,
        goal: &str,
        now_unix_ms: u64,
    ) -> Result<Self, OpenResponsesApiError> {
        let mut state = if path.exists() {
            load_gateway_mission_state(path)?
        } else {
            Self {
                schema_version: GATEWAY_MISSION_SCHEMA_VERSION,
                mission_id: mission_id.to_string(),
                session_key: session_key.to_string(),
                response_id: response_id.to_string(),
                goal_summary: summarize_gateway_mission_text(goal),
                latest_output_summary: String::new(),
                status: GatewayMissionStatus::Running,
                created_unix_ms: now_unix_ms,
                updated_unix_ms: now_unix_ms,
                iteration_count: 0,
                latest_verifier: GatewayMissionVerifierRecord {
                    kind: "mission_start".to_string(),
                    status: GatewayMissionVerifierStatus::Continue,
                    reason_code: "mission_started".to_string(),
                    message: "mission execution started".to_string(),
                    details: BTreeMap::new(),
                },
                latest_completion: None,
                iterations: Vec::new(),
            }
        };
        state.schema_version = GATEWAY_MISSION_SCHEMA_VERSION;
        state.mission_id = mission_id.to_string();
        state.session_key = session_key.to_string();
        state.response_id = response_id.to_string();
        state.goal_summary = summarize_gateway_mission_text(goal);
        state.status = GatewayMissionStatus::Running;
        state.updated_unix_ms = now_unix_ms;
        Ok(state)
    }

    pub(super) fn record_iteration(
        &mut self,
        attempt: usize,
        prompt: &str,
        assistant_summary: &str,
        tool_execution_count: usize,
        verifier: GatewayMissionVerifierBundle,
        completion: Option<GatewayMissionCompletionSignalRecord>,
        started_unix_ms: u64,
        finished_unix_ms: u64,
    ) {
        self.iterations.push(GatewayMissionIterationRecord {
            attempt,
            prompt_summary: summarize_gateway_mission_text(prompt),
            assistant_summary: summarize_gateway_mission_text(assistant_summary),
            tool_execution_count,
            verifier: verifier.clone(),
            completion: completion.clone(),
            started_unix_ms,
            finished_unix_ms,
        });
        if self.iterations.len() > GATEWAY_MISSION_MAX_ITERATIONS {
            let drop_count = self
                .iterations
                .len()
                .saturating_sub(GATEWAY_MISSION_MAX_ITERATIONS);
            self.iterations.drain(0..drop_count);
        }
        self.iteration_count = self.iterations.len();
        self.latest_verifier = verifier.overall;
        self.latest_completion = completion;
        self.updated_unix_ms = finished_unix_ms;
    }

    pub(super) fn mark_completed(
        &mut self,
        verifier: GatewayMissionVerifierRecord,
        completion: Option<GatewayMissionCompletionSignalRecord>,
        latest_output_summary: &str,
        updated_unix_ms: u64,
    ) {
        self.status = GatewayMissionStatus::Completed;
        self.latest_verifier = verifier;
        self.latest_completion = completion;
        self.latest_output_summary = summarize_gateway_mission_text(latest_output_summary);
        self.updated_unix_ms = updated_unix_ms;
    }

    pub(super) fn mark_checkpointed(
        &mut self,
        verifier: GatewayMissionVerifierRecord,
        completion: GatewayMissionCompletionSignalRecord,
        latest_output_summary: &str,
        updated_unix_ms: u64,
    ) {
        self.status = GatewayMissionStatus::Checkpointed;
        self.latest_verifier = verifier;
        self.latest_completion = Some(completion);
        self.latest_output_summary = summarize_gateway_mission_text(latest_output_summary);
        self.updated_unix_ms = updated_unix_ms;
    }

    pub(super) fn mark_blocked(
        &mut self,
        verifier: GatewayMissionVerifierRecord,
        completion: Option<GatewayMissionCompletionSignalRecord>,
        latest_output_summary: &str,
        updated_unix_ms: u64,
    ) {
        self.status = GatewayMissionStatus::Blocked;
        self.latest_verifier = verifier;
        self.latest_completion = completion;
        self.latest_output_summary = summarize_gateway_mission_text(latest_output_summary);
        self.updated_unix_ms = updated_unix_ms;
    }
}

pub(super) fn gateway_mission_state_path(state_dir: &Path, mission_id: &str) -> PathBuf {
    gateway_missions_root(state_dir).join(format!("{mission_id}.json"))
}

pub(super) fn gateway_missions_root(state_dir: &Path) -> PathBuf {
    state_dir.join("openresponses").join("missions")
}

pub(super) fn load_gateway_mission_state(
    path: &Path,
) -> Result<GatewayMissionState, OpenResponsesApiError> {
    let raw = std::fs::read_to_string(path).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to read mission supervisor state '{}': {error}",
            path.display()
        ))
    })?;
    serde_json::from_str::<GatewayMissionState>(&raw).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to parse mission supervisor state '{}': {error}",
            path.display()
        ))
    })
}

pub(super) fn save_gateway_mission_state(
    path: &Path,
    state: &GatewayMissionState,
) -> Result<(), OpenResponsesApiError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            OpenResponsesApiError::internal(format!(
                "failed to create mission supervisor directory '{}': {error}",
                parent.display()
            ))
        })?;
    }
    let serialized = serde_json::to_string_pretty(state).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to serialize mission supervisor state: {error}"
        ))
    })?;
    write_text_atomic(path, serialized.as_str()).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to persist mission supervisor state '{}': {error}",
            path.display()
        ))
    })
}

fn summarize_gateway_mission_text(raw: &str) -> String {
    let summary = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::new();
    for ch in summary.chars().take(GATEWAY_MISSION_SUMMARY_MAX_CHARS) {
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_gateway_mission_state_path_uses_mission_directory() {
        let path = gateway_mission_state_path(Path::new("/tmp/tau-gateway"), "mission-alpha");
        assert!(path.ends_with("openresponses/missions/mission-alpha.json"));
    }

    #[test]
    fn unit_summarize_gateway_mission_text_normalizes_whitespace_and_truncates() {
        let summary = summarize_gateway_mission_text(&format!("hello   world {}", "x".repeat(400)));
        assert!(summary.starts_with("hello world"));
        assert!(summary.chars().count() <= GATEWAY_MISSION_SUMMARY_MAX_CHARS);
    }
}
