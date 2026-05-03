//! Gateway-local mission supervisor persistence for the first Tau Ralph loop slice.

use super::*;
use tau_agent_core::{
    MissionCheckpoint, MissionCompletion, MissionCompletionStatus, MissionLifecycleStatus,
    MissionRecoveryState, MissionSnapshot, MissionVerificationGate, MissionVerifierRecord,
    MissionVerifierStatus,
};

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
    #[serde(default)]
    pub(super) request_payload: Value,
    #[serde(default)]
    pub(super) response_payload: Value,
    pub(super) verifier: GatewayMissionVerifierBundle,
    #[serde(default)]
    pub(super) completion: Option<GatewayMissionCompletionSignalRecord>,
    pub(super) started_unix_ms: u64,
    pub(super) finished_unix_ms: u64,
}

pub(super) struct GatewayMissionIterationInput<'a> {
    pub(super) attempt: usize,
    pub(super) prompt: &'a str,
    pub(super) assistant_summary: &'a str,
    pub(super) tool_execution_count: usize,
    pub(super) request_payload: Value,
    pub(super) response_payload: Value,
    pub(super) verifier: GatewayMissionVerifierBundle,
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

    pub(super) fn record_iteration(&mut self, input: GatewayMissionIterationInput<'_>) {
        self.iterations.push(GatewayMissionIterationRecord {
            attempt: input.attempt,
            prompt_summary: summarize_gateway_mission_text(input.prompt),
            assistant_summary: summarize_gateway_mission_text(input.assistant_summary),
            tool_execution_count: input.tool_execution_count,
            request_payload: input.request_payload,
            response_payload: input.response_payload,
            verifier: input.verifier.clone(),
            completion: input.completion.clone(),
            started_unix_ms: input.started_unix_ms,
            finished_unix_ms: input.finished_unix_ms,
        });
        if self.iterations.len() > GATEWAY_MISSION_MAX_ITERATIONS {
            let drop_count = self
                .iterations
                .len()
                .saturating_sub(GATEWAY_MISSION_MAX_ITERATIONS);
            self.iterations.drain(0..drop_count);
        }
        self.iteration_count = self.iterations.len();
        self.latest_verifier = input.verifier.overall;
        self.latest_completion = input.completion;
        self.updated_unix_ms = input.finished_unix_ms;
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

    pub(super) fn to_shared_mission_snapshot(&self) -> MissionSnapshot {
        let mut snapshot =
            MissionSnapshot::new(&self.mission_id, &self.goal_summary, self.created_unix_ms);
        snapshot.session_key = Some(self.session_key.clone());
        snapshot.response_id = Some(self.response_id.clone());
        snapshot.latest_output_summary = self.latest_output_summary.clone();
        snapshot.status = shared_status_from_gateway(&self.status);
        snapshot.updated_unix_ms = self.updated_unix_ms;
        snapshot.iteration_count = self.iteration_count;
        snapshot.tool_budget.consumed_tool_calls = self
            .iterations
            .iter()
            .map(|iteration| iteration.tool_execution_count)
            .sum();
        snapshot.latest_verifier = Some(shared_verifier_from_gateway(&self.latest_verifier));
        snapshot.latest_completion = self
            .latest_completion
            .as_ref()
            .map(shared_completion_from_gateway);
        snapshot.verification_gates.push(MissionVerificationGate {
            id: self.latest_verifier.kind.clone(),
            description: self.latest_verifier.message.clone(),
            status: Some(shared_verifier_status_from_gateway(
                &self.latest_verifier.status,
            )),
            evidence: self.latest_verifier.details.clone(),
        });

        if self.status == GatewayMissionStatus::Checkpointed {
            let next_step = self
                .latest_completion
                .as_ref()
                .and_then(|completion| completion.next_step.clone());
            snapshot.checkpoints.push(MissionCheckpoint {
                checkpoint_id: format!("{}:checkpoint:{}", self.mission_id, self.iteration_count),
                summary: checkpoint_summary(self),
                created_unix_ms: self.updated_unix_ms,
                pending_plan_node_ids: next_step.into_iter().collect(),
            });
        }

        if self.status == GatewayMissionStatus::Blocked {
            snapshot.recovery_state = Some(MissionRecoveryState {
                reason: checkpoint_summary(self),
                next_action: self
                    .latest_completion
                    .as_ref()
                    .and_then(|completion| completion.next_step.clone()),
                retry_count: self.iteration_count,
                last_checkpoint_id: None,
            });
        }

        snapshot
    }
}

fn shared_status_from_gateway(status: &GatewayMissionStatus) -> MissionLifecycleStatus {
    match status {
        GatewayMissionStatus::Running => MissionLifecycleStatus::Executing,
        GatewayMissionStatus::Completed => MissionLifecycleStatus::Completed,
        GatewayMissionStatus::Checkpointed => MissionLifecycleStatus::Checkpointed,
        GatewayMissionStatus::Blocked => MissionLifecycleStatus::Blocked,
    }
}

fn shared_verifier_status_from_gateway(
    status: &GatewayMissionVerifierStatus,
) -> MissionVerifierStatus {
    match status {
        GatewayMissionVerifierStatus::Passed => MissionVerifierStatus::Passed,
        GatewayMissionVerifierStatus::Continue => MissionVerifierStatus::Continue,
        GatewayMissionVerifierStatus::Failed => MissionVerifierStatus::Failed,
    }
}

fn shared_verifier_from_gateway(record: &GatewayMissionVerifierRecord) -> MissionVerifierRecord {
    MissionVerifierRecord {
        kind: record.kind.clone(),
        status: shared_verifier_status_from_gateway(&record.status),
        reason_code: record.reason_code.clone(),
        message: record.message.clone(),
        details: record.details.clone(),
    }
}

fn shared_completion_from_gateway(
    completion: &GatewayMissionCompletionSignalRecord,
) -> MissionCompletion {
    MissionCompletion {
        status: match &completion.status {
            GatewayMissionCompletionStatus::Success => MissionCompletionStatus::Success,
            GatewayMissionCompletionStatus::Partial => MissionCompletionStatus::Partial,
            GatewayMissionCompletionStatus::Blocked => MissionCompletionStatus::Blocked,
        },
        summary: completion.summary.clone(),
        next_step: completion.next_step.clone(),
    }
}

fn checkpoint_summary(state: &GatewayMissionState) -> String {
    if !state.latest_output_summary.is_empty() {
        return state.latest_output_summary.clone();
    }
    if let Some(completion) = &state.latest_completion {
        return completion.summary.clone();
    }
    state.latest_verifier.message.clone()
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

    #[test]
    fn regression_openresponses_attempt_traces_capture_request_and_response_payloads() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = gateway_mission_state_path(temp.path(), "mission-payloads");
        let mut state = GatewayMissionState::load_or_create(
            &path,
            "mission-payloads",
            "session-alpha",
            "resp_alpha",
            "run payload trace",
            100,
        )
        .expect("mission state");
        let verifier =
            GatewayMissionVerifierBundle::from_records(vec![GatewayMissionVerifierRecord {
                kind: "runtime".to_string(),
                status: GatewayMissionVerifierStatus::Passed,
                reason_code: "ok".to_string(),
                message: "ok".to_string(),
                details: BTreeMap::new(),
            }]);

        state.record_iteration(GatewayMissionIterationInput {
            attempt: 1,
            prompt: "run payload trace",
            assistant_summary: "payload trace complete",
            tool_execution_count: 1,
            request_payload: json!({
                "attempt": 1,
                "prompt": "run payload trace",
                "messages_before": [],
            }),
            response_payload: json!({
                "status": "completed",
                "messages": [{"role": "assistant", "content": [{"type": "text", "text": "payload trace complete"}]}],
                "tool_executions": [{"tool_call_id": "call-1", "tool_name": "read_file"}],
            }),
            verifier,
            completion: None,
            started_unix_ms: 110,
            finished_unix_ms: 120,
        });
        save_gateway_mission_state(&path, &state).expect("save mission state");

        let loaded = load_gateway_mission_state(&path).expect("load mission state");
        let iteration = loaded.iterations.first().expect("iteration record");
        assert_eq!(iteration.request_payload["prompt"], "run payload trace");
        assert_eq!(iteration.response_payload["status"], "completed");
        assert_eq!(
            iteration.response_payload["tool_executions"][0]["tool_call_id"],
            "call-1"
        );
    }

    #[test]
    fn gateway_mission_state_projects_to_shared_snapshot() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = gateway_mission_state_path(temp.path(), "mission-shared");
        let mut state = GatewayMissionState::load_or_create(
            &path,
            "mission-shared",
            "session-alpha",
            "resp_alpha",
            "ship the harness contract",
            100,
        )
        .expect("mission state");
        let verifier =
            GatewayMissionVerifierBundle::from_records(vec![GatewayMissionVerifierRecord {
                kind: "validation_evidence".to_string(),
                status: GatewayMissionVerifierStatus::Passed,
                reason_code: "validation_evidence_observed".to_string(),
                message: "validation passed".to_string(),
                details: BTreeMap::from([("command".to_string(), json!("cargo test"))]),
            }]);
        state.record_iteration(GatewayMissionIterationInput {
            attempt: 1,
            prompt: "ship the harness contract",
            assistant_summary: "checkpointed with tests",
            tool_execution_count: 2,
            request_payload: json!({"prompt": "ship the harness contract"}),
            response_payload: json!({"status": "completed"}),
            verifier: verifier.clone(),
            completion: Some(GatewayMissionCompletionSignalRecord {
                status: GatewayMissionCompletionStatus::Partial,
                summary: "checkpointed shared projection".to_string(),
                next_step: Some("resume adapter migration".to_string()),
            }),
            started_unix_ms: 110,
            finished_unix_ms: 120,
        });
        state.mark_checkpointed(
            verifier.overall,
            GatewayMissionCompletionSignalRecord {
                status: GatewayMissionCompletionStatus::Partial,
                summary: "checkpointed shared projection".to_string(),
                next_step: Some("resume adapter migration".to_string()),
            },
            "shared projection ready",
            130,
        );

        let shared = state.to_shared_mission_snapshot();

        assert_eq!(shared.mission_id, "mission-shared");
        assert_eq!(shared.session_key.as_deref(), Some("session-alpha"));
        assert_eq!(shared.response_id.as_deref(), Some("resp_alpha"));
        assert_eq!(shared.goal, "ship the harness contract");
        assert_eq!(shared.status, MissionLifecycleStatus::Checkpointed);
        assert_eq!(shared.iteration_count, 1);
        assert_eq!(shared.tool_budget.consumed_tool_calls, 2);
        assert_eq!(
            shared.latest_verifier.as_ref().map(|record| record.status),
            Some(MissionVerifierStatus::Passed)
        );
        assert_eq!(
            shared
                .latest_completion
                .as_ref()
                .map(|record| record.status),
            Some(MissionCompletionStatus::Partial)
        );
        assert_eq!(shared.verification_gates.len(), 1);
        assert_eq!(shared.checkpoints.len(), 1);
        assert_eq!(
            shared.checkpoints[0].pending_plan_node_ids,
            vec!["resume adapter migration".to_string()]
        );
    }
}
