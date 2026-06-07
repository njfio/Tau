use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    MissionArtifactRef, MissionCuratorReviewStatus, MissionLearningRecord,
    MissionLearningRecordKind, MissionLifecycleStatus, MissionSnapshot, MissionTransitionError,
    MissionVerificationGate, MissionVerifierRecord, MissionVerifierStatus,
};

pub const CODING_MISSION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingMissionPhase {
    Intake,
    Planned,
    PreparingBranch,
    Executing,
    Verifying,
    PrReady,
    Blocked,
    Completed,
}

impl CodingMissionPhase {
    fn mission_status(self) -> MissionLifecycleStatus {
        match self {
            Self::Intake => MissionLifecycleStatus::Draft,
            Self::Planned => MissionLifecycleStatus::Planned,
            Self::PreparingBranch | Self::Executing => MissionLifecycleStatus::Executing,
            Self::Verifying => MissionLifecycleStatus::Verifying,
            Self::PrReady => MissionLifecycleStatus::Checkpointed,
            Self::Blocked => MissionLifecycleStatus::Blocked,
            Self::Completed => MissionLifecycleStatus::Completed,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingMissionPrMode {
    Disabled,
    PrReady,
    Draft,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionConfig {
    pub state_root: PathBuf,
    pub mission_id: String,
    pub session_key: String,
    pub repo_path: PathBuf,
    #[serde(default)]
    pub issue_url: Option<String>,
    pub goal: String,
    pub base_branch: String,
    pub branch_prefix: String,
    pub verifier_commands: Vec<String>,
    pub pr_mode: CodingMissionPrMode,
    pub allowed_roots: Vec<PathBuf>,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionEvent {
    pub phase: CodingMissionPhase,
    pub reason_code: String,
    pub message: String,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodingMissionState {
    pub schema_version: u32,
    pub state_root: PathBuf,
    pub mission_id: String,
    pub session_key: String,
    pub repo_path: PathBuf,
    #[serde(default)]
    pub issue_url: Option<String>,
    pub goal: String,
    pub base_branch: String,
    pub branch_prefix: String,
    pub verifier_commands: Vec<String>,
    pub pr_mode: CodingMissionPrMode,
    pub allowed_roots: Vec<PathBuf>,
    pub phase: CodingMissionPhase,
    pub created_unix_ms: u64,
    pub updated_unix_ms: u64,
    pub mission: MissionSnapshot,
    #[serde(default)]
    pub events: Vec<CodingMissionEvent>,
}

impl CodingMissionState {
    pub fn create(config: CodingMissionConfig) -> Result<Self, CodingMissionError> {
        validate_required_token("mission_id", &config.mission_id)?;
        validate_path_token("mission_id", &config.mission_id)?;
        validate_required_token("session_key", &config.session_key)?;
        validate_required_token("goal", &config.goal)?;
        validate_required_token("base_branch", &config.base_branch)?;
        if config.verifier_commands.is_empty() {
            return Err(CodingMissionError::InvalidConfig {
                field: "verifier_commands",
                message: "at least one verifier command is required".to_string(),
            });
        }

        let repo_path = canonicalize_existing_dir("repo_path", &config.repo_path)?;
        let allowed_roots = canonicalize_allowed_roots(&config.allowed_roots)?;
        if !allowed_roots.iter().any(|root| repo_path.starts_with(root)) {
            return Err(CodingMissionError::RepoOutsideAllowedRoots {
                repo_path,
                allowed_roots,
            });
        }

        let mut mission =
            MissionSnapshot::new(&config.mission_id, &config.goal, config.created_unix_ms);
        mission.session_key = Some(config.session_key.clone());
        mission.artifacts.push(MissionArtifactRef {
            artifact_id: "workspace".to_string(),
            kind: "coding_workspace".to_string(),
            path: Some(repo_path.display().to_string()),
            summary: Some("coding mission workspace root".to_string()),
        });
        mission.verification_gates = config
            .verifier_commands
            .iter()
            .map(|command| MissionVerificationGate {
                id: format!("verifier:{command}"),
                description: format!("Verifier command: {command}"),
                status: None,
                evidence: BTreeMap::from([("command".to_string(), json!(command))]),
            })
            .collect();
        mission.learning_records.push(MissionLearningRecord {
            record_id: format!("{}:coding-mission-state", config.mission_id),
            mission_id: config.mission_id.clone(),
            kind: MissionLearningRecordKind::Final,
            summary: "Coding mission state is attached to mission proof fields".to_string(),
            created_unix_ms: config.created_unix_ms,
            curator_status: MissionCuratorReviewStatus::QueuedForReview,
            root_cause: None,
            evidence: vec!["coding mission state initialized".to_string()],
            artifact_ids: vec!["workspace".to_string()],
            verification_gate_ids: mission
                .verification_gates
                .iter()
                .map(|gate| gate.id.clone())
                .collect(),
            rollback_plan: Some("delete the coding mission state file".to_string()),
            metadata: BTreeMap::from([
                ("base_branch".to_string(), json!(config.base_branch)),
                ("branch_prefix".to_string(), json!(config.branch_prefix)),
                ("pr_mode".to_string(), json!(config.pr_mode)),
                ("issue_url".to_string(), json!(config.issue_url)),
            ]),
        });
        mission.latest_output_summary = "coding mission intake initialized".to_string();
        mission.latest_verifier = Some(coding_mission_verifier_record(
            CodingMissionPhase::Intake,
            "coding_mission_created",
            "coding mission state initialized",
            BTreeMap::from([
                (
                    "repo_path".to_string(),
                    json!(repo_path.display().to_string()),
                ),
                (
                    "verifier_command_count".to_string(),
                    json!(mission.verification_gates.len()),
                ),
            ]),
        ));

        let mut state = Self {
            schema_version: CODING_MISSION_SCHEMA_VERSION,
            state_root: config.state_root,
            mission_id: config.mission_id,
            session_key: config.session_key,
            repo_path,
            issue_url: config.issue_url,
            goal: config.goal,
            base_branch: config.base_branch,
            branch_prefix: config.branch_prefix,
            verifier_commands: config.verifier_commands,
            pr_mode: config.pr_mode,
            allowed_roots,
            phase: CodingMissionPhase::Intake,
            created_unix_ms: config.created_unix_ms,
            updated_unix_ms: config.created_unix_ms,
            mission,
            events: Vec::new(),
        };
        state.events.push(CodingMissionEvent {
            phase: CodingMissionPhase::Intake,
            reason_code: "coding_mission_created".to_string(),
            message: "coding mission state initialized".to_string(),
            created_unix_ms: state.created_unix_ms,
        });
        Ok(state)
    }

    pub fn transition_phase(
        &mut self,
        next: CodingMissionPhase,
        reason_code: impl Into<String>,
        message: impl Into<String>,
        updated_unix_ms: u64,
    ) -> Result<(), CodingMissionError> {
        let next_status = next.mission_status();
        self.mission
            .transition_to(next_status, updated_unix_ms)
            .map_err(CodingMissionError::MissionTransition)?;
        let reason_code = reason_code.into();
        let message = message.into();
        self.phase = next;
        self.updated_unix_ms = updated_unix_ms;
        self.mission.latest_output_summary = message.clone();
        self.mission.latest_verifier = Some(coding_mission_verifier_record(
            next,
            reason_code.as_str(),
            message.as_str(),
            BTreeMap::new(),
        ));
        self.events.push(CodingMissionEvent {
            phase: next,
            reason_code,
            message,
            created_unix_ms: updated_unix_ms,
        });
        Ok(())
    }

    pub fn to_mission_snapshot(&self) -> MissionSnapshot {
        self.mission.clone()
    }
}

fn coding_mission_verifier_record(
    phase: CodingMissionPhase,
    reason_code: &str,
    message: &str,
    details: BTreeMap<String, serde_json::Value>,
) -> MissionVerifierRecord {
    MissionVerifierRecord {
        kind: "coding_mission_state".to_string(),
        status: match phase {
            CodingMissionPhase::Completed => MissionVerifierStatus::Passed,
            CodingMissionPhase::Blocked => MissionVerifierStatus::Failed,
            _ => MissionVerifierStatus::Continue,
        },
        reason_code: reason_code.to_string(),
        message: message.to_string(),
        details,
    }
}

#[derive(Debug, Error)]
pub enum CodingMissionError {
    #[error("invalid coding mission config field {field}: {message}")]
    InvalidConfig {
        field: &'static str,
        message: String,
    },
    #[error("repo path {repo_path} is outside allowed roots {allowed_roots:?}")]
    RepoOutsideAllowedRoots {
        repo_path: PathBuf,
        allowed_roots: Vec<PathBuf>,
    },
    #[error("failed to read coding mission state {path}: {source}")]
    StateRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse coding mission state {path}: {source}")]
    StateParse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported coding mission schema_version {actual} in {path}; expected {expected}")]
    UnsupportedSchema {
        path: PathBuf,
        actual: u32,
        expected: u32,
    },
    #[error("failed to serialize coding mission state: {source}")]
    StateSerialize { source: serde_json::Error },
    #[error("failed to persist coding mission state {path}: {source}")]
    StateWrite {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid coding mission transition: {0}")]
    MissionTransition(#[from] MissionTransitionError),
}

pub fn coding_mission_state_path(state_root: &Path, mission_id: &str) -> PathBuf {
    state_root
        .join("coding-missions")
        .join(format!("{mission_id}.json"))
}

pub fn save_coding_mission_state(state: &CodingMissionState) -> Result<(), CodingMissionError> {
    let path = coding_mission_state_path(&state.state_root, &state.mission_id);
    let serialized = serde_json::to_string_pretty(state)
        .map_err(|source| CodingMissionError::StateSerialize { source })?;
    write_text_atomic(path.as_path(), serialized.as_str()).map_err(|source| {
        CodingMissionError::StateWrite {
            path: path.clone(),
            source,
        }
    })
}

pub fn load_coding_mission_state(
    state_root: &Path,
    mission_id: &str,
) -> Result<CodingMissionState, CodingMissionError> {
    let path = coding_mission_state_path(state_root, mission_id);
    let raw = std::fs::read_to_string(&path).map_err(|source| CodingMissionError::StateRead {
        path: path.clone(),
        source,
    })?;
    let state = serde_json::from_str::<CodingMissionState>(&raw).map_err(|source| {
        CodingMissionError::StateParse {
            path: path.clone(),
            source,
        }
    })?;
    if state.schema_version != CODING_MISSION_SCHEMA_VERSION {
        return Err(CodingMissionError::UnsupportedSchema {
            path,
            actual: state.schema_version,
            expected: CODING_MISSION_SCHEMA_VERSION,
        });
    }
    Ok(state)
}

fn validate_required_token(field: &'static str, value: &str) -> Result<(), CodingMissionError> {
    if value.trim().is_empty() {
        return Err(CodingMissionError::InvalidConfig {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_path_token(field: &'static str, value: &str) -> Result<(), CodingMissionError> {
    if value.contains('/') || value.contains('\\') || value.contains("..") {
        return Err(CodingMissionError::InvalidConfig {
            field,
            message: "must not contain path separators or parent traversal".to_string(),
        });
    }
    Ok(())
}

fn canonicalize_existing_dir(
    field: &'static str,
    path: &Path,
) -> Result<PathBuf, CodingMissionError> {
    let canonical =
        std::fs::canonicalize(path).map_err(|error| CodingMissionError::InvalidConfig {
            field,
            message: format!("failed to canonicalize {}: {error}", path.display()),
        })?;
    if !canonical.is_dir() {
        return Err(CodingMissionError::InvalidConfig {
            field,
            message: format!("{} is not a directory", canonical.display()),
        });
    }
    Ok(canonical)
}

fn canonicalize_allowed_roots(roots: &[PathBuf]) -> Result<Vec<PathBuf>, CodingMissionError> {
    if roots.is_empty() {
        return Err(CodingMissionError::InvalidConfig {
            field: "allowed_roots",
            message: "at least one allowed root is required".to_string(),
        });
    }
    roots
        .iter()
        .map(|root| canonicalize_existing_dir("allowed_roots", root))
        .collect()
}

fn write_text_atomic(path: &Path, payload: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("coding-mission-state");
    let tmp_path = path.with_file_name(format!("{file_name}.tmp.{}", std::process::id()));
    std::fs::write(&tmp_path, payload)?;
    std::fs::rename(tmp_path, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn config_for(root: &std::path::Path) -> CodingMissionConfig {
        let repo = root.join("repo");
        std::fs::create_dir_all(&repo).expect("repo dir");
        CodingMissionConfig {
            state_root: root.join("state"),
            mission_id: "coding-mission-alpha".to_string(),
            session_key: "session-alpha".to_string(),
            repo_path: repo.clone(),
            issue_url: Some("https://github.com/njfio/Tau/issues/3654".to_string()),
            goal: "implement the coding mission state contract".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "tau/".to_string(),
            verifier_commands: vec!["cargo test -p tau-agent-core coding_mission".to_string()],
            pr_mode: CodingMissionPrMode::PrReady,
            allowed_roots: vec![root.to_path_buf()],
            created_unix_ms: 1_800_000_000_000,
        }
    }

    #[test]
    fn spec_c01_coding_mission_create_persist_loads_typed_state() {
        let temp = tempdir().expect("tempdir");
        let config = config_for(temp.path());

        let mut state = CodingMissionState::create(config).expect("create state");
        state
            .transition_phase(
                CodingMissionPhase::Planned,
                "coding_mission_planned",
                "plan created",
                1_800_000_000_100,
            )
            .expect("planned");
        save_coding_mission_state(&state).expect("save state");

        let loaded =
            load_coding_mission_state(state.state_root.as_path(), state.mission_id.as_str())
                .expect("load state");

        assert_eq!(loaded.schema_version, CODING_MISSION_SCHEMA_VERSION);
        assert_eq!(loaded.phase, CodingMissionPhase::Planned);
        assert_eq!(loaded.events.len(), 2);
        assert_eq!(loaded.verifier_commands, state.verifier_commands);
        assert!(
            coding_mission_state_path(&loaded.state_root, &loaded.mission_id)
                .ends_with("coding-missions/coding-mission-alpha.json")
        );
    }

    #[test]
    fn spec_c02_coding_mission_state_projects_to_shared_mission_snapshot() {
        let temp = tempdir().expect("tempdir");
        let state = CodingMissionState::create(config_for(temp.path())).expect("create state");
        let mission = state.to_mission_snapshot();

        assert_eq!(mission.mission_id, state.mission_id);
        assert_eq!(mission.session_key.as_deref(), Some("session-alpha"));
        assert_eq!(mission.goal, state.goal);
        assert_eq!(mission.artifacts.len(), 1);
        assert_eq!(mission.artifacts[0].artifact_id, "workspace");
        assert_eq!(mission.artifacts[0].kind, "coding_workspace");
        assert_eq!(
            mission
                .latest_verifier
                .as_ref()
                .map(|record| record.reason_code.as_str()),
            Some("coding_mission_created")
        );
        assert_eq!(mission.learning_records.len(), 1);
        assert_eq!(
            mission.learning_records[0].record_id,
            "coding-mission-alpha:coding-mission-state"
        );
        assert_eq!(mission.learning_records[0].artifact_ids, vec!["workspace"]);
        assert_eq!(
            mission.verification_gates[0].id,
            "verifier:cargo test -p tau-agent-core coding_mission"
        );
        assert_eq!(
            mission.learning_records[0].verification_gate_ids,
            vec!["verifier:cargo test -p tau-agent-core coding_mission"]
        );
    }

    #[test]
    fn regression_coding_mission_rejects_repo_outside_allowed_roots() {
        let temp = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let repo = outside.path().join("repo");
        std::fs::create_dir_all(&repo).expect("repo dir");
        let mut config = config_for(temp.path());
        config.repo_path = repo;

        let error = CodingMissionState::create(config).expect_err("out of root should fail");

        assert!(matches!(
            error,
            CodingMissionError::RepoOutsideAllowedRoots { .. }
        ));
    }

    #[test]
    fn regression_load_coding_mission_state_fails_closed_on_corrupt_json() {
        let temp = tempdir().expect("tempdir");
        let path = coding_mission_state_path(temp.path(), "corrupt-mission");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
        std::fs::write(&path, "{not-json").expect("write corrupt state");

        let error = load_coding_mission_state(temp.path(), "corrupt-mission")
            .expect_err("corrupt JSON should fail");

        assert!(matches!(error, CodingMissionError::StateParse { .. }));
    }

    #[test]
    fn regression_load_coding_mission_state_rejects_unsupported_schema_version() {
        let temp = tempdir().expect("tempdir");
        let state = CodingMissionState::create(config_for(temp.path())).expect("create state");
        save_coding_mission_state(&state).expect("save state");
        let path = coding_mission_state_path(&state.state_root, &state.mission_id);
        let mut payload = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&path).expect("read state"),
        )
        .expect("parse saved state");
        payload["schema_version"] = serde_json::json!(999);
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&payload).expect("serialize unsupported schema"),
        )
        .expect("write unsupported schema");

        let error = load_coding_mission_state(&state.state_root, &state.mission_id)
            .expect_err("unsupported schema should fail");

        assert!(matches!(
            error,
            CodingMissionError::UnsupportedSchema { .. }
        ));
    }
}
