use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    process::Command,
    time::Instant,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    MissionArtifactRef, MissionCuratorReviewStatus, MissionLearningRecord,
    MissionLearningRecordKind, MissionLifecycleStatus, MissionSnapshot, MissionToolCallEvidence,
    MissionToolCallStatus, MissionToolEvidenceError, MissionTransitionError,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingWorkspaceCommandPolicy {
    pub allowed_roots: Vec<PathBuf>,
    pub allow_network: bool,
    pub allow_mutation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingWorkspaceCommand {
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    pub reason_code: String,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingWorkspaceCommandStatus {
    Succeeded,
    Failed,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingWorkspaceCommandEvidence {
    pub command_id: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub exit_status: Option<i32>,
    pub elapsed_ms: u64,
    pub reason_code: String,
    pub status: CodingWorkspaceCommandStatus,
    #[serde(default)]
    pub denied_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingGitLifecycleEvidenceKind {
    BranchPrepared,
    CommitCreated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingGitLifecycleEvidence {
    pub kind: CodingGitLifecycleEvidenceKind,
    pub branch_name: String,
    pub base_branch: String,
    pub created_branch: bool,
    pub reused_branch: bool,
    #[serde(default)]
    pub commit_hash: Option<String>,
    #[serde(default)]
    pub changed_files: Vec<String>,
    pub reason_code: String,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingGitPrepareBranchRequest {
    #[serde(default)]
    pub branch_name: Option<String>,
    pub allow_fetch: bool,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingGitCommitRequest {
    pub message: String,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionControlledEdit {
    pub relative_path: PathBuf,
    pub contents: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionRunRequest {
    #[serde(default)]
    pub controlled_edit: Option<CodingMissionControlledEdit>,
    #[serde(default)]
    pub controlled_edits: Vec<CodingMissionControlledEdit>,
    pub commit_message: String,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionRunOutcome {
    pub phase: CodingMissionPhase,
    pub verifier_passed: bool,
    #[serde(default)]
    pub committed: Option<CodingGitLifecycleEvidence>,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    pub iterations: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingMissionResumeAction {
    RunVerifier,
    ApplyEdit,
    VerifyAfterEdit,
    Commit,
    Blocked,
    PrReady,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingMissionResumeStopAfter {
    ControlledEdit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionMutationFingerprint {
    pub changed_files: Vec<String>,
    pub diff_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionResumeCheckpoint {
    pub next_action: CodingMissionResumeAction,
    #[serde(default)]
    pub branch_name: Option<String>,
    #[serde(default)]
    pub pending_verifier_command: Option<String>,
    #[serde(default)]
    pub latest_verifier_command_id: Option<String>,
    #[serde(default)]
    pub mutation_fingerprint: Option<CodingMissionMutationFingerprint>,
    pub latest_learning_summary: String,
    pub operator_resume_command: String,
    pub updated_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionResumeRequest {
    pub state_root: PathBuf,
    pub mission_id: String,
    #[serde(default)]
    pub controlled_edit: Option<CodingMissionControlledEdit>,
    #[serde(default)]
    pub controlled_edits: Vec<CodingMissionControlledEdit>,
    pub commit_message: String,
    pub started_unix_ms: u64,
    #[serde(default)]
    pub stop_after: Option<CodingMissionResumeStopAfter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodingMissionResumeOutcome {
    pub state: CodingMissionState,
    pub run: CodingMissionRunOutcome,
    pub resume_action: CodingMissionResumeAction,
    #[serde(default)]
    pub restored_branch: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodingMissionPrPublicationStatus {
    ManualReady,
    DraftCreated,
    DraftFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionPrReadyRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub risk_notes: Vec<String>,
    #[serde(default)]
    pub rollback_notes: Vec<String>,
    pub allow_draft_pr: bool,
    #[serde(default)]
    pub github_env: BTreeMap<String, String>,
    #[serde(default)]
    pub gh_binary: Option<PathBuf>,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingMissionPrReadyBundle {
    pub status: CodingMissionPrPublicationStatus,
    pub branch_name: String,
    #[serde(default)]
    pub commit_hash: Option<String>,
    pub title: String,
    pub body: String,
    pub body_path: PathBuf,
    pub manual_gh_pr_create_command: String,
    pub changed_files: Vec<String>,
    pub verifier_evidence_ids: Vec<String>,
    pub risk_notes: Vec<String>,
    pub rollback_notes: Vec<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub error_summary: Option<String>,
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
    #[serde(default)]
    pub command_evidence: Vec<CodingWorkspaceCommandEvidence>,
    #[serde(default)]
    pub git_evidence: Vec<CodingGitLifecycleEvidence>,
    #[serde(default)]
    pub resume_checkpoint: Option<CodingMissionResumeCheckpoint>,
    #[serde(default)]
    pub pr_ready_bundle: Option<CodingMissionPrReadyBundle>,
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
            command_evidence: Vec::new(),
            git_evidence: Vec::new(),
            resume_checkpoint: None,
            pr_ready_bundle: None,
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

    pub fn prepare_branch(
        &mut self,
        request: CodingGitPrepareBranchRequest,
    ) -> Result<CodingGitLifecycleEvidence, CodingMissionError> {
        let branch_name = request
            .branch_name
            .unwrap_or_else(|| default_coding_git_branch_name(self));
        if request.allow_fetch {
            let fetch = execute_git_command(
                self,
                vec!["git", "fetch", "--all", "--prune"],
                "git_fetch_opt_in",
                request.started_unix_ms,
                false,
                true,
            )?;
            if fetch.status != CodingWorkspaceCommandStatus::Succeeded {
                return Err(CodingMissionError::GitLifecycle {
                    reason_code: "fetch_failed",
                    message: command_evidence_summary(&fetch),
                });
            }
        }

        let status = execute_git_command(
            self,
            vec!["git", "status", "--porcelain"],
            "git_status_dirty_check",
            request.started_unix_ms.saturating_add(1),
            false,
            false,
        )?;
        let dirty_status = command_stdout(&status)?;
        if !dirty_status.trim().is_empty() {
            return Err(CodingMissionError::GitLifecycle {
                reason_code: "dirty_tree",
                message: dirty_status,
            });
        }

        let current = execute_git_command(
            self,
            vec!["git", "branch", "--show-current"],
            "git_current_branch",
            request.started_unix_ms.saturating_add(2),
            false,
            false,
        )?;
        let current_branch = command_stdout(&current)?.trim().to_string();
        let exists = execute_git_command(
            self,
            vec![
                "git".to_string(),
                "show-ref".to_string(),
                "--verify".to_string(),
                "--quiet".to_string(),
                format!("refs/heads/{branch_name}"),
            ],
            "git_branch_exists",
            request.started_unix_ms.saturating_add(3),
            false,
            false,
        )?;
        let branch_exists = exists.status == CodingWorkspaceCommandStatus::Succeeded;

        let (created_branch, reused_branch) = if current_branch == branch_name {
            (false, true)
        } else {
            if current_branch != self.base_branch {
                return Err(CodingMissionError::GitLifecycle {
                    reason_code: "base_branch_mismatch",
                    message: format!(
                        "current branch {current_branch} does not match base {}",
                        self.base_branch
                    ),
                });
            }
            if branch_exists {
                let switched = execute_git_command(
                    self,
                    vec!["git", "switch", branch_name.as_str()],
                    "git_switch_existing_branch",
                    request.started_unix_ms.saturating_add(4),
                    true,
                    false,
                )?;
                if switched.status != CodingWorkspaceCommandStatus::Succeeded {
                    return Err(CodingMissionError::GitLifecycle {
                        reason_code: "branch_switch_failed",
                        message: command_evidence_summary(&switched),
                    });
                }
                (false, true)
            } else {
                let created = execute_git_command(
                    self,
                    vec!["git", "switch", "-c", branch_name.as_str()],
                    "git_switch_create_branch",
                    request.started_unix_ms.saturating_add(4),
                    true,
                    false,
                )?;
                if created.status != CodingWorkspaceCommandStatus::Succeeded {
                    return Err(CodingMissionError::GitLifecycle {
                        reason_code: "branch_create_failed",
                        message: command_evidence_summary(&created),
                    });
                }
                (true, false)
            }
        };

        let head = execute_git_command(
            self,
            vec!["git", "rev-parse", "HEAD"],
            "git_branch_head",
            request.started_unix_ms.saturating_add(5),
            false,
            false,
        )?;
        let evidence = CodingGitLifecycleEvidence {
            kind: CodingGitLifecycleEvidenceKind::BranchPrepared,
            branch_name,
            base_branch: self.base_branch.clone(),
            created_branch,
            reused_branch,
            commit_hash: Some(command_stdout(&head)?.trim().to_string()),
            changed_files: Vec::new(),
            reason_code: "branch_prepared".to_string(),
            created_unix_ms: request.started_unix_ms.saturating_add(5),
        };
        self.git_evidence.push(evidence.clone());
        save_coding_mission_state(self)?;
        Ok(evidence)
    }

    pub fn commit_changes(
        &mut self,
        request: CodingGitCommitRequest,
    ) -> Result<CodingGitLifecycleEvidence, CodingMissionError> {
        let current = execute_git_command(
            self,
            vec!["git", "branch", "--show-current"],
            "git_commit_current_branch",
            request.started_unix_ms,
            false,
            false,
        )?;
        let branch_name = command_stdout(&current)?.trim().to_string();
        let status = execute_git_command(
            self,
            vec!["git", "status", "--porcelain"],
            "git_commit_status",
            request.started_unix_ms.saturating_add(1),
            false,
            false,
        )?;
        let changed_files = parse_git_status_changed_files(&command_stdout(&status)?);
        if changed_files.is_empty() {
            return Err(CodingMissionError::GitLifecycle {
                reason_code: "missing_git_diff",
                message: "no changed files available to commit".to_string(),
            });
        }

        let add = execute_git_command(
            self,
            vec!["git", "add", "-A"],
            "git_add_all",
            request.started_unix_ms.saturating_add(2),
            true,
            false,
        )?;
        if add.status != CodingWorkspaceCommandStatus::Succeeded {
            return Err(CodingMissionError::GitLifecycle {
                reason_code: "git_add_failed",
                message: command_evidence_summary(&add),
            });
        }

        let (subject, body) = coding_git_commit_message(self, &request.message);
        let commit = execute_git_command(
            self,
            vec!["git", "commit", "-m", subject.as_str(), "-m", body.as_str()],
            "git_commit",
            request.started_unix_ms.saturating_add(3),
            true,
            false,
        )?;
        if commit.status != CodingWorkspaceCommandStatus::Succeeded {
            return Err(CodingMissionError::GitLifecycle {
                reason_code: "git_commit_failed",
                message: command_evidence_summary(&commit),
            });
        }
        let hash = execute_git_command(
            self,
            vec!["git", "rev-parse", "HEAD"],
            "git_commit_hash",
            request.started_unix_ms.saturating_add(4),
            false,
            false,
        )?;
        let evidence = CodingGitLifecycleEvidence {
            kind: CodingGitLifecycleEvidenceKind::CommitCreated,
            branch_name,
            base_branch: self.base_branch.clone(),
            created_branch: false,
            reused_branch: true,
            commit_hash: Some(command_stdout(&hash)?.trim().to_string()),
            changed_files,
            reason_code: "commit_created".to_string(),
            created_unix_ms: request.started_unix_ms.saturating_add(4),
        };
        self.git_evidence.push(evidence.clone());
        save_coding_mission_state(self)?;
        Ok(evidence)
    }

    pub fn prepare_pr_ready_bundle(
        &mut self,
        request: CodingMissionPrReadyRequest,
    ) -> Result<CodingMissionPrReadyBundle, CodingMissionError> {
        if self.phase != CodingMissionPhase::PrReady {
            return Err(CodingMissionError::GitLifecycle {
                reason_code: "mission_not_pr_ready",
                message: format!("mission phase is {:?}", self.phase),
            });
        }
        let commit = latest_commit_evidence(self)?;
        let title = request.title.clone().unwrap_or_else(|| {
            format!(
                "Coding mission {}: {}",
                self.mission_id,
                first_non_empty_line(&self.goal)
            )
        });
        let verifier_evidence_ids = self
            .command_evidence
            .iter()
            .filter(|evidence| evidence.reason_code.starts_with("coding_verifier"))
            .map(|evidence| evidence.command_id.clone())
            .collect::<Vec<_>>();
        let body = render_pr_ready_body(self, &commit, &request, &verifier_evidence_ids);
        let body_path = coding_mission_pr_body_path(&self.state_root, &self.mission_id);
        write_text_atomic(&body_path, &body).map_err(|source| CodingMissionError::StateWrite {
            path: body_path.clone(),
            source,
        })?;
        let manual_command = render_manual_gh_pr_create_command(
            commit.branch_name.as_str(),
            title.as_str(),
            body_path.as_path(),
        );
        let mut bundle = CodingMissionPrReadyBundle {
            status: CodingMissionPrPublicationStatus::ManualReady,
            branch_name: commit.branch_name.clone(),
            commit_hash: commit.commit_hash.clone(),
            title,
            body,
            body_path: body_path.clone(),
            manual_gh_pr_create_command: manual_command,
            changed_files: commit.changed_files.clone(),
            verifier_evidence_ids,
            risk_notes: request.risk_notes.clone(),
            rollback_notes: request.rollback_notes.clone(),
            pr_url: None,
            error_summary: None,
            created_unix_ms: request.started_unix_ms,
        };
        self.mission.artifacts.push(MissionArtifactRef {
            artifact_id: "pr-ready-body".to_string(),
            kind: "coding_mission_pr_body".to_string(),
            path: Some(body_path.display().to_string()),
            summary: Some("PR-ready bundle body".to_string()),
        });

        if request.allow_draft_pr && github_auth_present(&request.github_env) {
            publish_draft_pr(self, &request, &mut bundle)?;
        }

        self.pr_ready_bundle = Some(bundle.clone());
        self.updated_unix_ms = request.started_unix_ms;
        self.mission.latest_output_summary = match bundle.status {
            CodingMissionPrPublicationStatus::DraftCreated => {
                format!(
                    "draft PR created: {}",
                    bundle.pr_url.as_deref().unwrap_or("url_missing")
                )
            }
            CodingMissionPrPublicationStatus::DraftFailed => {
                format!(
                    "draft PR creation failed: {}",
                    bundle.error_summary.as_deref().unwrap_or("unknown")
                )
            }
            CodingMissionPrPublicationStatus::ManualReady => {
                "PR-ready bundle prepared for manual gh pr create".to_string()
            }
        };
        self.events.push(CodingMissionEvent {
            phase: self.phase,
            reason_code: "pr_ready_bundle_prepared".to_string(),
            message: self.mission.latest_output_summary.clone(),
            created_unix_ms: request.started_unix_ms,
        });
        save_coding_mission_state(self)?;
        Ok(bundle)
    }
}

fn latest_commit_evidence(
    state: &CodingMissionState,
) -> Result<CodingGitLifecycleEvidence, CodingMissionError> {
    state
        .git_evidence
        .iter()
        .rev()
        .find(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated)
        .cloned()
        .ok_or_else(|| CodingMissionError::GitLifecycle {
            reason_code: "missing_commit_evidence",
            message: "PR-ready bundle requires commit evidence".to_string(),
        })
}

fn render_pr_ready_body(
    state: &CodingMissionState,
    commit: &CodingGitLifecycleEvidence,
    request: &CodingMissionPrReadyRequest,
    verifier_evidence_ids: &[String],
) -> String {
    let changed_files = if commit.changed_files.is_empty() {
        "- none recorded".to_string()
    } else {
        commit
            .changed_files
            .iter()
            .map(|file| format!("- {file}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let verifier_evidence = if verifier_evidence_ids.is_empty() {
        "- none recorded".to_string()
    } else {
        verifier_evidence_ids
            .iter()
            .map(|id| format!("- {id}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let risk_notes = render_note_lines(&request.risk_notes, "None");
    let rollback_notes = render_note_lines(&request.rollback_notes, "Revert the mission commit");
    format!(
        "Summary:\nCoding mission is ready for PR publication.\n\nMission: {}\nSession: {}\nGoal: {}\n\nBranch: {}\nCommit: {}\n\nChanged files:\n{}\n\nVerifier evidence:\n{}\n\nRisks:\n{}\n\nRollback:\n{}\n",
        state.mission_id,
        state.session_key,
        state.goal,
        commit.branch_name,
        commit.commit_hash.as_deref().unwrap_or("missing"),
        changed_files,
        verifier_evidence,
        risk_notes,
        rollback_notes
    )
}

fn render_note_lines(notes: &[String], empty: &str) -> String {
    if notes.is_empty() {
        format!("- {empty}")
    } else {
        notes
            .iter()
            .map(|note| format!("- {note}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn first_non_empty_line(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("coding mission")
        .to_string()
}

fn render_manual_gh_pr_create_command(branch_name: &str, title: &str, body_path: &Path) -> String {
    format!(
        "gh pr create --draft --head {} --title {} --body-file {}",
        shell_quote(branch_name),
        shell_quote(title),
        shell_quote(body_path.display().to_string().as_str())
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn coding_mission_pr_body_path(state_root: &Path, mission_id: &str) -> PathBuf {
    state_root
        .join("coding-missions")
        .join(mission_id)
        .join("pr-ready-body.md")
}

fn github_auth_present(env: &BTreeMap<String, String>) -> bool {
    env.get("GH_TOKEN")
        .or_else(|| env.get("GITHUB_TOKEN"))
        .is_some_and(|value| !value.trim().is_empty())
        || std::env::var("GH_TOKEN").is_ok_and(|value| !value.trim().is_empty())
        || std::env::var("GITHUB_TOKEN").is_ok_and(|value| !value.trim().is_empty())
}

fn publish_draft_pr(
    state: &mut CodingMissionState,
    request: &CodingMissionPrReadyRequest,
    bundle: &mut CodingMissionPrReadyBundle,
) -> Result<(), CodingMissionError> {
    let gh_binary = request
        .gh_binary
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "gh".to_string());
    let body_path = bundle.body_path.display().to_string();
    let evidence = execute_workspace_command(
        state,
        vec![
            gh_binary,
            "pr".to_string(),
            "create".to_string(),
            "--draft".to_string(),
            "--head".to_string(),
            bundle.branch_name.clone(),
            "--title".to_string(),
            bundle.title.clone(),
            "--body-file".to_string(),
            body_path,
        ],
        "gh_pr_create_draft".to_string(),
        request.started_unix_ms.saturating_add(1),
        false,
        true,
    )?;
    if evidence.status == CodingWorkspaceCommandStatus::Succeeded {
        let stdout = command_stdout(&evidence)?;
        bundle.pr_url = stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string);
        bundle.status = CodingMissionPrPublicationStatus::DraftCreated;
    } else {
        bundle.status = CodingMissionPrPublicationStatus::DraftFailed;
        bundle.error_summary = Some(command_evidence_summary(&evidence));
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct CodingMissionRunner;

impl CodingMissionRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(
        &self,
        state: &mut CodingMissionState,
        request: CodingMissionRunRequest,
    ) -> Result<CodingMissionRunOutcome, CodingMissionError> {
        let controlled_edits =
            requested_controlled_edits(&request.controlled_edit, &request.controlled_edits);
        if state.phase == CodingMissionPhase::Intake {
            state.transition_phase(
                CodingMissionPhase::Planned,
                "coding_mission_planned",
                "coding mission runner planned branch and verifier work",
                request.started_unix_ms,
            )?;
            save_coding_mission_state(state)?;
        }

        let branch = state.prepare_branch(CodingGitPrepareBranchRequest {
            branch_name: None,
            allow_fetch: false,
            started_unix_ms: request.started_unix_ms.saturating_add(10),
        })?;
        state.transition_phase(
            CodingMissionPhase::Executing,
            "coding_mission_executing",
            "coding mission runner executing verifier loop",
            request.started_unix_ms.saturating_add(20),
        )?;
        save_coding_mission_state(state)?;
        record_resume_checkpoint(
            state,
            CodingMissionResumeAction::RunVerifier,
            Some(branch.branch_name.clone()),
            None,
            "resume_run_verifier",
            request.started_unix_ms.saturating_add(30),
        )?;

        let mut iterations = 1;
        let first =
            run_coding_mission_verifiers(state, request.started_unix_ms.saturating_add(100))?;
        if let Some(reason) = first.blocked_reason {
            return block_coding_mission_run(
                state,
                reason,
                first.verifier_passed,
                iterations,
                request.started_unix_ms.saturating_add(199),
            );
        }
        let mut verifier_passed = first.verifier_passed;
        if !verifier_passed {
            if !controlled_edits.is_empty() {
                if let Err(error) = apply_controlled_edits_and_checkpoint(
                    state,
                    &controlled_edits,
                    branch.branch_name.as_str(),
                    request.started_unix_ms.saturating_add(200),
                ) {
                    return block_controlled_edit_error(
                        state,
                        error,
                        false,
                        iterations,
                        request.started_unix_ms.saturating_add(299),
                    );
                }
                iterations = iterations.saturating_add(1);
                let second = run_coding_mission_verifiers(
                    state,
                    request.started_unix_ms.saturating_add(300),
                )?;
                if let Some(reason) = second.blocked_reason {
                    return block_coding_mission_run(
                        state,
                        reason,
                        second.verifier_passed,
                        iterations,
                        request.started_unix_ms.saturating_add(399),
                    );
                }
                verifier_passed = second.verifier_passed;
            } else {
                record_resume_checkpoint(
                    state,
                    CodingMissionResumeAction::ApplyEdit,
                    Some(branch.branch_name),
                    None,
                    "resume_apply_edit",
                    request.started_unix_ms.saturating_add(200),
                )?;
                return Ok(CodingMissionRunOutcome {
                    phase: state.phase,
                    verifier_passed: false,
                    committed: None,
                    blocked_reason: None,
                    iterations,
                });
            }
        }

        if !verifier_passed {
            return Ok(CodingMissionRunOutcome {
                phase: state.phase,
                verifier_passed,
                committed: None,
                blocked_reason: None,
                iterations,
            });
        }

        let commit_branch_name = latest_prepared_branch_name(state);
        let commit_fingerprint =
            current_mutation_fingerprint(state, request.started_unix_ms.saturating_add(390)).ok();
        record_resume_checkpoint(
            state,
            CodingMissionResumeAction::Commit,
            commit_branch_name,
            commit_fingerprint,
            "resume_commit",
            request.started_unix_ms.saturating_add(399),
        )?;
        commit_verified_coding_mission(
            state,
            request.commit_message,
            iterations,
            request.started_unix_ms.saturating_add(400),
        )
    }

    pub fn resume(
        &self,
        request: CodingMissionResumeRequest,
    ) -> Result<CodingMissionResumeOutcome, CodingMissionError> {
        let mut state = load_coding_mission_state(&request.state_root, &request.mission_id)?;
        let checkpoint = match state.resume_checkpoint.clone() {
            Some(checkpoint) => checkpoint,
            None => {
                let run = block_coding_mission_run(
                    &mut state,
                    "missing_resume_checkpoint",
                    false,
                    0,
                    request.started_unix_ms,
                )?;
                return Ok(CodingMissionResumeOutcome {
                    state,
                    run,
                    resume_action: CodingMissionResumeAction::Blocked,
                    restored_branch: None,
                });
            }
        };
        let restored_branch =
            restore_resume_branch(&mut state, &checkpoint, request.started_unix_ms)?;
        let resume_action = checkpoint.next_action;
        let run = match resume_action {
            CodingMissionResumeAction::ApplyEdit => {
                resume_from_apply_edit(&mut state, &checkpoint, request)?
            }
            CodingMissionResumeAction::VerifyAfterEdit => {
                resume_from_verify_after_edit(&mut state, &checkpoint, request)?
            }
            CodingMissionResumeAction::Commit => {
                resume_from_commit(&mut state, &checkpoint, request)?
            }
            CodingMissionResumeAction::RunVerifier => {
                resume_from_run_verifier(&mut state, &checkpoint, request)?
            }
            CodingMissionResumeAction::Blocked => CodingMissionRunOutcome {
                phase: state.phase,
                verifier_passed: false,
                committed: None,
                blocked_reason: Some("mission_blocked".to_string()),
                iterations: 0,
            },
            CodingMissionResumeAction::PrReady => CodingMissionRunOutcome {
                phase: state.phase,
                verifier_passed: true,
                committed: state
                    .git_evidence
                    .iter()
                    .rev()
                    .find(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated)
                    .cloned(),
                blocked_reason: None,
                iterations: 0,
            },
        };
        Ok(CodingMissionResumeOutcome {
            state,
            run,
            resume_action,
            restored_branch,
        })
    }
}

fn resume_from_run_verifier(
    state: &mut CodingMissionState,
    checkpoint: &CodingMissionResumeCheckpoint,
    request: CodingMissionResumeRequest,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    let branch_name = checkpoint
        .branch_name
        .clone()
        .or_else(|| latest_prepared_branch_name(state));
    let first = run_coding_mission_verifiers(state, request.started_unix_ms.saturating_add(100))?;
    if let Some(reason) = first.blocked_reason {
        return block_coding_mission_run(
            state,
            reason,
            first.verifier_passed,
            1,
            request.started_unix_ms.saturating_add(199),
        );
    }
    if first.verifier_passed {
        let mutation_fingerprint =
            current_mutation_fingerprint(state, request.started_unix_ms.saturating_add(190)).ok();
        record_resume_checkpoint(
            state,
            CodingMissionResumeAction::Commit,
            branch_name,
            mutation_fingerprint,
            "resume_commit",
            request.started_unix_ms.saturating_add(199),
        )?;
        return commit_verified_coding_mission(
            state,
            request.commit_message,
            1,
            request.started_unix_ms.saturating_add(200),
        );
    }
    record_resume_checkpoint(
        state,
        CodingMissionResumeAction::ApplyEdit,
        branch_name,
        None,
        "resume_apply_edit",
        request.started_unix_ms.saturating_add(200),
    )?;
    Ok(CodingMissionRunOutcome {
        phase: state.phase,
        verifier_passed: false,
        committed: None,
        blocked_reason: None,
        iterations: 1,
    })
}

fn resume_from_apply_edit(
    state: &mut CodingMissionState,
    checkpoint: &CodingMissionResumeCheckpoint,
    request: CodingMissionResumeRequest,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    let controlled_edits =
        requested_controlled_edits(&request.controlled_edit, &request.controlled_edits);
    if controlled_edits.is_empty() {
        return Ok(CodingMissionRunOutcome {
            phase: state.phase,
            verifier_passed: false,
            committed: None,
            blocked_reason: None,
            iterations: 0,
        });
    }
    let branch_name = checkpoint
        .branch_name
        .clone()
        .or_else(|| latest_prepared_branch_name(state))
        .unwrap_or_else(|| default_coding_git_branch_name(state));
    if let Err(error) = apply_controlled_edits_and_checkpoint(
        state,
        &controlled_edits,
        branch_name.as_str(),
        request.started_unix_ms.saturating_add(100),
    ) {
        return block_controlled_edit_error(
            state,
            error,
            false,
            1,
            request.started_unix_ms.saturating_add(199),
        );
    }
    if request.stop_after == Some(CodingMissionResumeStopAfter::ControlledEdit) {
        return Ok(CodingMissionRunOutcome {
            phase: state.phase,
            verifier_passed: false,
            committed: None,
            blocked_reason: None,
            iterations: 1,
        });
    }
    let checkpoint = state
        .resume_checkpoint
        .clone()
        .expect("controlled edit checkpoint");
    resume_from_verify_after_edit(state, &checkpoint, request)
}

fn resume_from_verify_after_edit(
    state: &mut CodingMissionState,
    checkpoint: &CodingMissionResumeCheckpoint,
    request: CodingMissionResumeRequest,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    if let Some(expected) = checkpoint.mutation_fingerprint.as_ref() {
        let current =
            current_mutation_fingerprint(state, request.started_unix_ms.saturating_add(150))?;
        if &current != expected {
            return block_coding_mission_run(
                state,
                "mutation_fingerprint_mismatch",
                false,
                1,
                request.started_unix_ms.saturating_add(199),
            );
        }
    }
    let second = run_coding_mission_verifiers(state, request.started_unix_ms.saturating_add(200))?;
    if let Some(reason) = second.blocked_reason {
        return block_coding_mission_run(
            state,
            reason,
            second.verifier_passed,
            2,
            request.started_unix_ms.saturating_add(299),
        );
    }
    if !second.verifier_passed {
        record_resume_checkpoint(
            state,
            CodingMissionResumeAction::ApplyEdit,
            checkpoint
                .branch_name
                .clone()
                .or_else(|| latest_prepared_branch_name(state)),
            None,
            "resume_apply_edit_after_failed_verify",
            request.started_unix_ms.saturating_add(300),
        )?;
        return Ok(CodingMissionRunOutcome {
            phase: state.phase,
            verifier_passed: false,
            committed: None,
            blocked_reason: None,
            iterations: 2,
        });
    }
    let commit_branch_name = checkpoint
        .branch_name
        .clone()
        .or_else(|| latest_prepared_branch_name(state));
    let commit_fingerprint =
        current_mutation_fingerprint(state, request.started_unix_ms.saturating_add(300)).ok();
    record_resume_checkpoint(
        state,
        CodingMissionResumeAction::Commit,
        commit_branch_name,
        commit_fingerprint,
        "resume_commit",
        request.started_unix_ms.saturating_add(301),
    )?;
    commit_verified_coding_mission(
        state,
        request.commit_message,
        2,
        request.started_unix_ms.saturating_add(400),
    )
}

fn resume_from_commit(
    state: &mut CodingMissionState,
    checkpoint: &CodingMissionResumeCheckpoint,
    request: CodingMissionResumeRequest,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    if let Some(expected) = checkpoint.mutation_fingerprint.as_ref() {
        let current =
            current_mutation_fingerprint(state, request.started_unix_ms.saturating_add(100))?;
        if &current != expected {
            return block_coding_mission_run(
                state,
                "mutation_fingerprint_mismatch",
                true,
                1,
                request.started_unix_ms.saturating_add(199),
            );
        }
    }
    commit_verified_coding_mission(
        state,
        request.commit_message,
        1,
        request.started_unix_ms.saturating_add(200),
    )
}

fn commit_verified_coding_mission(
    state: &mut CodingMissionState,
    commit_message: String,
    iterations: usize,
    started_unix_ms: u64,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    match state.commit_changes(CodingGitCommitRequest {
        message: commit_message,
        started_unix_ms,
    }) {
        Ok(commit) => {
            state.transition_phase(
                CodingMissionPhase::PrReady,
                "coding_mission_pr_ready",
                "verifier passed and mission-linked commit evidence is ready",
                started_unix_ms.saturating_add(100),
            )?;
            record_resume_checkpoint(
                state,
                CodingMissionResumeAction::PrReady,
                Some(commit.branch_name.clone()),
                None,
                "resume_pr_ready",
                started_unix_ms.saturating_add(101),
            )?;
            save_coding_mission_state(state)?;
            Ok(CodingMissionRunOutcome {
                phase: state.phase,
                verifier_passed: true,
                committed: Some(commit),
                blocked_reason: None,
                iterations,
            })
        }
        Err(CodingMissionError::GitLifecycle {
            reason_code,
            message: _,
        }) => block_coding_mission_run(
            state,
            reason_code,
            true,
            iterations,
            started_unix_ms.saturating_add(100),
        ),
        Err(error) => Err(error),
    }
}

fn restore_resume_branch(
    state: &mut CodingMissionState,
    checkpoint: &CodingMissionResumeCheckpoint,
    started_unix_ms: u64,
) -> Result<Option<String>, CodingMissionError> {
    let Some(branch_name) = checkpoint
        .branch_name
        .clone()
        .or_else(|| latest_prepared_branch_name(state))
    else {
        return Ok(None);
    };
    let current = execute_git_command(
        state,
        vec!["git", "branch", "--show-current"],
        "resume_current_branch",
        started_unix_ms,
        false,
        false,
    )?;
    let current_branch = command_stdout(&current)?.trim().to_string();
    if current_branch == branch_name {
        return Ok(Some(branch_name));
    }
    let switched = execute_git_command(
        state,
        vec!["git", "switch", branch_name.as_str()],
        "resume_switch_branch",
        started_unix_ms.saturating_add(1),
        true,
        false,
    )?;
    if switched.status != CodingWorkspaceCommandStatus::Succeeded {
        return Err(CodingMissionError::GitLifecycle {
            reason_code: "resume_branch_restore_failed",
            message: command_evidence_summary(&switched),
        });
    }
    Ok(Some(branch_name))
}

fn requested_controlled_edits(
    controlled_edit: &Option<CodingMissionControlledEdit>,
    controlled_edits: &[CodingMissionControlledEdit],
) -> Vec<CodingMissionControlledEdit> {
    let mut edits = Vec::with_capacity(
        usize::from(controlled_edit.is_some()).saturating_add(controlled_edits.len()),
    );
    if let Some(edit) = controlled_edit.clone() {
        edits.push(edit);
    }
    edits.extend(controlled_edits.iter().cloned());
    edits
}

fn apply_controlled_edits_and_checkpoint(
    state: &mut CodingMissionState,
    edits: &[CodingMissionControlledEdit],
    branch_name: &str,
    started_unix_ms: u64,
) -> Result<(), CodingMissionError> {
    apply_controlled_edits(state, edits, started_unix_ms)?;
    let fingerprint = current_mutation_fingerprint(state, started_unix_ms.saturating_add(1))?;
    record_resume_checkpoint(
        state,
        CodingMissionResumeAction::VerifyAfterEdit,
        Some(branch_name.to_string()),
        Some(fingerprint),
        "resume_verify_after_edit",
        started_unix_ms.saturating_add(2),
    )
}

fn block_controlled_edit_error(
    state: &mut CodingMissionState,
    error: CodingMissionError,
    verifier_passed: bool,
    iterations: usize,
    updated_unix_ms: u64,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    match error {
        CodingMissionError::GitLifecycle {
            reason_code: "controlled_edit_outside_repo",
            message: _,
        } => block_coding_mission_run(
            state,
            "controlled_edit_outside_repo",
            verifier_passed,
            iterations,
            updated_unix_ms,
        ),
        other => Err(other),
    }
}

fn record_resume_checkpoint(
    state: &mut CodingMissionState,
    next_action: CodingMissionResumeAction,
    branch_name: Option<String>,
    mutation_fingerprint: Option<CodingMissionMutationFingerprint>,
    reason_code: &'static str,
    updated_unix_ms: u64,
) -> Result<(), CodingMissionError> {
    let latest_verifier_command_id = state
        .command_evidence
        .iter()
        .rev()
        .find(|evidence| evidence.reason_code.starts_with("coding_verifier"))
        .map(|evidence| evidence.command_id.clone());
    let pending_verifier_command = state.verifier_commands.first().cloned();
    let latest_learning_summary = state.mission.latest_output_summary.clone();
    let operator_resume_command = format!("mission resume {}", state.mission_id);
    state.resume_checkpoint = Some(CodingMissionResumeCheckpoint {
        next_action,
        branch_name,
        pending_verifier_command,
        latest_verifier_command_id,
        mutation_fingerprint,
        latest_learning_summary,
        operator_resume_command,
        updated_unix_ms,
    });
    state.updated_unix_ms = updated_unix_ms;
    let message = format!("resume checkpoint saved for {:?}", next_action);
    if next_action != CodingMissionResumeAction::PrReady {
        state.mission.latest_output_summary = message.clone();
    }
    state.mission.latest_verifier = Some(coding_mission_verifier_record(
        state.phase,
        reason_code,
        message.as_str(),
        BTreeMap::from([
            ("next_action".to_string(), json!(next_action)),
            (
                "operator_resume_command".to_string(),
                json!(format!("mission resume {}", state.mission_id)),
            ),
        ]),
    ));
    state.events.push(CodingMissionEvent {
        phase: state.phase,
        reason_code: reason_code.to_string(),
        message,
        created_unix_ms: updated_unix_ms,
    });
    save_coding_mission_state(state)
}

fn latest_prepared_branch_name(state: &CodingMissionState) -> Option<String> {
    state
        .git_evidence
        .iter()
        .rev()
        .find(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::BranchPrepared)
        .map(|evidence| evidence.branch_name.clone())
}

fn current_mutation_fingerprint(
    state: &mut CodingMissionState,
    started_unix_ms: u64,
) -> Result<CodingMissionMutationFingerprint, CodingMissionError> {
    let status = execute_git_command(
        state,
        vec!["git", "status", "--porcelain"],
        "git_mutation_fingerprint_status",
        started_unix_ms,
        false,
        false,
    )?;
    let raw_status = command_stdout(&status)?;
    let changed_files = parse_git_status_changed_files(&raw_status);
    let mut payload = Vec::new();
    payload.extend_from_slice(raw_status.as_bytes());
    for file in &changed_files {
        payload.push(0);
        payload.extend_from_slice(file.as_bytes());
        let path = normalize_path_lexically(&state.repo_path.join(file));
        if path.starts_with(&state.repo_path) && path.is_file() {
            let contents =
                std::fs::read(&path).map_err(|source| CodingMissionError::StateRead {
                    path: path.clone(),
                    source,
                })?;
            payload.push(0);
            payload.extend_from_slice(&contents);
        }
    }
    Ok(CodingMissionMutationFingerprint {
        changed_files,
        diff_hash: stable_fingerprint_hex(&payload),
    })
}

fn stable_fingerprint_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

struct CodingVerifierRun {
    verifier_passed: bool,
    blocked_reason: Option<&'static str>,
}

fn run_coding_mission_verifiers(
    state: &mut CodingMissionState,
    started_unix_ms: u64,
) -> Result<CodingVerifierRun, CodingMissionError> {
    let verifier_commands = state.verifier_commands.clone();
    if verifier_commands.is_empty() {
        return Ok(CodingVerifierRun {
            verifier_passed: false,
            blocked_reason: Some("missing_verifier_commands"),
        });
    }

    let mut all_passed = true;
    for (index, command) in verifier_commands.iter().enumerate() {
        let argv = split_command_argv(command);
        if argv.is_empty() {
            return Ok(CodingVerifierRun {
                verifier_passed: false,
                blocked_reason: Some("empty_verifier_command"),
            });
        }
        let evidence = execute_workspace_command(
            state,
            argv,
            format!("coding_verifier_{:04}", index.saturating_add(1)),
            started_unix_ms.saturating_add(index as u64),
            false,
            false,
        )?;
        match evidence.status {
            CodingWorkspaceCommandStatus::Succeeded => {}
            CodingWorkspaceCommandStatus::Denied => {
                return Ok(CodingVerifierRun {
                    verifier_passed: false,
                    blocked_reason: Some("verifier_command_denied"),
                });
            }
            CodingWorkspaceCommandStatus::Failed if evidence.exit_status.is_none() => {
                return Ok(CodingVerifierRun {
                    verifier_passed: false,
                    blocked_reason: Some("verifier_command_failed_to_start"),
                });
            }
            CodingWorkspaceCommandStatus::Failed => {
                all_passed = false;
            }
        }
    }
    Ok(CodingVerifierRun {
        verifier_passed: all_passed,
        blocked_reason: None,
    })
}

fn apply_controlled_edits(
    state: &mut CodingMissionState,
    edits: &[CodingMissionControlledEdit],
    created_unix_ms: u64,
) -> Result<(), CodingMissionError> {
    let targets = validate_controlled_edit_targets(state, edits)?;
    for (edit, target) in edits.iter().zip(targets.iter()) {
        write_text_atomic(target, &edit.contents).map_err(|source| {
            CodingMissionError::StateWrite {
                path: target.clone(),
                source,
            }
        })?;
    }
    for (edit, target) in edits.iter().zip(targets.iter()) {
        let artifact_id = format!("controlled_edit:{}", edit.relative_path.display());
        state.mission.artifacts.push(MissionArtifactRef {
            artifact_id,
            kind: "coding_mission_controlled_edit".to_string(),
            path: Some(target.display().to_string()),
            summary: Some(edit.reason_code.clone()),
        });
        state.events.push(CodingMissionEvent {
            phase: state.phase,
            reason_code: edit.reason_code.clone(),
            message: format!("controlled edit wrote {}", edit.relative_path.display()),
            created_unix_ms,
        });
    }
    state.updated_unix_ms = created_unix_ms;
    state.mission.latest_output_summary = match edits {
        [edit] => format!("controlled edit wrote {}", edit.relative_path.display()),
        _ => format!("controlled edit set wrote {} files", edits.len()),
    };
    save_coding_mission_state(state)
}

fn validate_controlled_edit_targets(
    state: &CodingMissionState,
    edits: &[CodingMissionControlledEdit],
) -> Result<Vec<PathBuf>, CodingMissionError> {
    edits
        .iter()
        .map(|edit| validate_controlled_edit_target(state, edit))
        .collect()
}

fn validate_controlled_edit_target(
    state: &CodingMissionState,
    edit: &CodingMissionControlledEdit,
) -> Result<PathBuf, CodingMissionError> {
    if edit.relative_path.is_absolute()
        || edit.relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(CodingMissionError::GitLifecycle {
            reason_code: "controlled_edit_outside_repo",
            message: edit.relative_path.display().to_string(),
        });
    }
    let target = normalize_path_lexically(&state.repo_path.join(&edit.relative_path));
    if !target.starts_with(&state.repo_path) {
        return Err(CodingMissionError::GitLifecycle {
            reason_code: "controlled_edit_outside_repo",
            message: target.display().to_string(),
        });
    }
    Ok(target)
}

fn block_coding_mission_run(
    state: &mut CodingMissionState,
    reason: &'static str,
    verifier_passed: bool,
    iterations: usize,
    updated_unix_ms: u64,
) -> Result<CodingMissionRunOutcome, CodingMissionError> {
    state.transition_phase(
        CodingMissionPhase::Blocked,
        reason,
        format!("coding mission blocked: {reason}"),
        updated_unix_ms,
    )?;
    save_coding_mission_state(state)?;
    Ok(CodingMissionRunOutcome {
        phase: state.phase,
        verifier_passed,
        committed: None,
        blocked_reason: Some(reason.to_string()),
        iterations,
    })
}

fn split_command_argv(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

#[derive(Debug, Clone)]
pub struct CodingWorkspaceExecutor {
    policy: CodingWorkspaceCommandPolicy,
}

impl CodingWorkspaceExecutor {
    pub fn new(policy: CodingWorkspaceCommandPolicy) -> Self {
        Self { policy }
    }

    pub fn execute(
        &self,
        state: &mut CodingMissionState,
        command: CodingWorkspaceCommand,
    ) -> Result<CodingWorkspaceCommandEvidence, CodingMissionError> {
        let command_id = format!("cmd-{:04}", state.command_evidence.len().saturating_add(1));
        let stdout_path = coding_mission_command_artifact_path(
            &state.state_root,
            &state.mission_id,
            &command_id,
            "stdout",
        );
        let stderr_path = coding_mission_command_artifact_path(
            &state.state_root,
            &state.mission_id,
            &command_id,
            "stderr",
        );

        let cwd = match self.evaluate_policy(state, &command) {
            Ok(cwd) => cwd,
            Err(denied_reason) => {
                return record_workspace_command_evidence(
                    state,
                    WorkspaceCommandRecord {
                        command,
                        command_id,
                        stdout_path,
                        stderr_path,
                        status: CodingWorkspaceCommandStatus::Denied,
                        exit_status: None,
                        elapsed_ms: 0,
                        denied_reason: Some(denied_reason),
                        stdout: Vec::new(),
                        stderr: format!("{denied_reason}\n").into_bytes(),
                    },
                );
            }
        };

        record_workspace_command_start(state, &command_id, &command)?;
        let started = Instant::now();
        let output = Command::new(&command.argv[0])
            .args(command.argv.iter().skip(1))
            .current_dir(&cwd)
            .output();
        let elapsed_ms = elapsed_ms_saturating(started);

        match output {
            Ok(output) => {
                let status = if output.status.success() {
                    CodingWorkspaceCommandStatus::Succeeded
                } else {
                    CodingWorkspaceCommandStatus::Failed
                };
                record_workspace_command_evidence(
                    state,
                    WorkspaceCommandRecord {
                        command,
                        command_id,
                        stdout_path,
                        stderr_path,
                        status,
                        exit_status: output.status.code(),
                        elapsed_ms,
                        denied_reason: None,
                        stdout: output.stdout,
                        stderr: output.stderr,
                    },
                )
            }
            Err(error) => record_workspace_command_evidence(
                state,
                WorkspaceCommandRecord {
                    command,
                    command_id,
                    stdout_path,
                    stderr_path,
                    status: CodingWorkspaceCommandStatus::Failed,
                    exit_status: None,
                    elapsed_ms,
                    denied_reason: None,
                    stdout: Vec::new(),
                    stderr: format!("command_spawn_failed: {error}\n").into_bytes(),
                },
            ),
        }
    }

    fn evaluate_policy(
        &self,
        state: &CodingMissionState,
        command: &CodingWorkspaceCommand,
    ) -> Result<PathBuf, &'static str> {
        if command.argv.is_empty() {
            return Err("empty_argv");
        }
        if !command.cwd.exists() {
            return Err("cwd_missing");
        }
        let cwd = match std::fs::canonicalize(&command.cwd) {
            Ok(cwd) if cwd.is_dir() => cwd,
            Ok(_) => return Err("cwd_not_directory"),
            Err(_) => return Err("cwd_invalid"),
        };
        if !path_is_allowed_by_roots(&cwd, &self.policy.allowed_roots)
            || !path_is_allowed_by_roots(&cwd, &state.allowed_roots)
        {
            return Err("cwd_outside_allowed_roots");
        }
        if command_is_force_push(&command.argv) {
            return Err("denied_force_push");
        }
        if command_is_destructive(&command.argv) {
            return Err("denied_destructive_command");
        }
        if !self.policy.allow_network && command_uses_network(&command.argv) {
            return Err("network_denied");
        }
        if !self.policy.allow_mutation && command_requires_mutation(&command.argv) {
            return Err("mutation_phase_required");
        }
        if command_has_out_of_root_write(&cwd, &command.argv, &self.policy.allowed_roots)
            || command_has_out_of_root_write(&cwd, &command.argv, &state.allowed_roots)
        {
            return Err("out_of_root_write");
        }
        Ok(cwd)
    }
}

fn execute_git_command<I, S>(
    state: &mut CodingMissionState,
    argv: I,
    reason_code: &str,
    started_unix_ms: u64,
    allow_mutation: bool,
    allow_network: bool,
) -> Result<CodingWorkspaceCommandEvidence, CodingMissionError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    execute_workspace_command(
        state,
        argv,
        reason_code.to_string(),
        started_unix_ms,
        allow_mutation,
        allow_network,
    )
}

fn execute_workspace_command<I, S>(
    state: &mut CodingMissionState,
    argv: I,
    reason_code: String,
    started_unix_ms: u64,
    allow_mutation: bool,
    allow_network: bool,
) -> Result<CodingWorkspaceCommandEvidence, CodingMissionError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let executor = CodingWorkspaceExecutor::new(CodingWorkspaceCommandPolicy {
        allowed_roots: state.allowed_roots.clone(),
        allow_network,
        allow_mutation,
    });
    let cwd = state.repo_path.clone();
    executor.execute(
        state,
        CodingWorkspaceCommand {
            cwd,
            argv: argv.into_iter().map(Into::into).collect(),
            reason_code,
            started_unix_ms,
        },
    )
}

fn command_stdout(evidence: &CodingWorkspaceCommandEvidence) -> Result<String, CodingMissionError> {
    std::fs::read_to_string(&evidence.stdout_path).map_err(|source| CodingMissionError::StateRead {
        path: evidence.stdout_path.clone(),
        source,
    })
}

fn default_coding_git_branch_name(state: &CodingMissionState) -> String {
    format!("{}{}", state.branch_prefix, state.mission_id)
}

fn parse_git_status_changed_files(raw: &str) -> Vec<String> {
    let mut files = raw
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| {
            path.rsplit_once(" -> ")
                .map(|(_, renamed)| renamed)
                .unwrap_or(path)
                .trim_matches('"')
                .to_string()
        })
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn coding_git_commit_message(state: &CodingMissionState, message: &str) -> (String, String) {
    let subject = message
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .unwrap_or("Apply coding mission changes")
        .to_string();
    if repo_requires_lore_commit(&state.repo_path) {
        (
            subject,
            format!(
                "Mission: {}\nSession: {}\n\nConstraint: Coding mission commit must be linked to durable mission evidence\nConfidence: medium\nScope-risk: narrow\nDirective: Keep this commit linked to CodingMissionState evidence\nTested: verifier commands recorded in mission state\nNot-tested: PR publication in this lifecycle step",
                state.mission_id, state.session_key
            ),
        )
    } else {
        (
            subject,
            format!(
                "Mission: {}\nSession: {}",
                state.mission_id, state.session_key
            ),
        )
    }
}

fn repo_requires_lore_commit(repo_path: &Path) -> bool {
    std::fs::read_to_string(repo_path.join("AGENTS.md"))
        .map(|contents| contents.contains("Lore Commit Protocol"))
        .unwrap_or(false)
}

fn record_workspace_command_start(
    state: &mut CodingMissionState,
    command_id: &str,
    command: &CodingWorkspaceCommand,
) -> Result<(), CodingMissionError> {
    let started_path = coding_mission_command_artifact_path(
        &state.state_root,
        &state.mission_id,
        command_id,
        "started.json",
    );
    let payload = serde_json::to_string_pretty(&json!({
        "command_id": command_id,
        "cwd": command.cwd.display().to_string(),
        "argv": command.argv,
        "reason_code": command.reason_code,
        "started_unix_ms": command.started_unix_ms,
    }))
    .map_err(|source| CodingMissionError::StateSerialize { source })?;
    write_text_atomic(&started_path, &payload).map_err(|source| {
        CodingMissionError::StateWrite {
            path: started_path.clone(),
            source,
        }
    })?;
    state.mission.artifacts.push(MissionArtifactRef {
        artifact_id: format!("{command_id}:started"),
        kind: "workspace_command_started".to_string(),
        path: Some(started_path.display().to_string()),
        summary: Some(format!("started marker for {}", command.reason_code)),
    });
    state.updated_unix_ms = command.started_unix_ms;
    state.mission.latest_output_summary = format!("command {command_id} started");
    state.events.push(CodingMissionEvent {
        phase: state.phase,
        reason_code: "workspace_command_started".to_string(),
        message: format!("command {command_id} started: {}", command.reason_code),
        created_unix_ms: command.started_unix_ms,
    });
    save_coding_mission_state(state)
}

struct WorkspaceCommandRecord {
    command: CodingWorkspaceCommand,
    command_id: String,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    status: CodingWorkspaceCommandStatus,
    exit_status: Option<i32>,
    elapsed_ms: u64,
    denied_reason: Option<&'static str>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn record_workspace_command_evidence(
    state: &mut CodingMissionState,
    record: WorkspaceCommandRecord,
) -> Result<CodingWorkspaceCommandEvidence, CodingMissionError> {
    write_bytes_atomic(&record.stdout_path, &record.stdout).map_err(|source| {
        CodingMissionError::StateWrite {
            path: record.stdout_path.clone(),
            source,
        }
    })?;
    write_bytes_atomic(&record.stderr_path, &record.stderr).map_err(|source| {
        CodingMissionError::StateWrite {
            path: record.stderr_path.clone(),
            source,
        }
    })?;

    let evidence = CodingWorkspaceCommandEvidence {
        command_id: record.command_id.clone(),
        cwd: record.command.cwd,
        argv: record.command.argv,
        stdout_path: record.stdout_path.clone(),
        stderr_path: record.stderr_path.clone(),
        exit_status: record.exit_status,
        elapsed_ms: record.elapsed_ms,
        reason_code: record.command.reason_code,
        status: record.status,
        denied_reason: record.denied_reason.map(str::to_string),
    };
    let stdout_artifact_id = format!("{}:stdout", record.command_id);
    let stderr_artifact_id = format!("{}:stderr", record.command_id);
    state.mission.artifacts.push(MissionArtifactRef {
        artifact_id: stdout_artifact_id.clone(),
        kind: "workspace_command_stdout".to_string(),
        path: Some(record.stdout_path.display().to_string()),
        summary: Some(format!("stdout for {}", evidence.reason_code)),
    });
    state.mission.artifacts.push(MissionArtifactRef {
        artifact_id: stderr_artifact_id.clone(),
        kind: "workspace_command_stderr".to_string(),
        path: Some(record.stderr_path.display().to_string()),
        summary: Some(format!("stderr for {}", evidence.reason_code)),
    });

    let mission_status = match record.status {
        CodingWorkspaceCommandStatus::Succeeded => MissionToolCallStatus::Succeeded,
        CodingWorkspaceCommandStatus::Failed => MissionToolCallStatus::Failed,
        CodingWorkspaceCommandStatus::Denied => MissionToolCallStatus::Blocked,
    };
    let completed_unix_ms = record
        .command
        .started_unix_ms
        .saturating_add(evidence.elapsed_ms);
    let verification_gate_ids = matching_verification_gate_ids(state, &evidence.argv);
    state
        .mission
        .record_tool_call_evidence(MissionToolCallEvidence {
            tool_call_id: evidence.command_id.clone(),
            mission_id: state.mission_id.clone(),
            plan_node_id: None,
            tool_name: "coding_workspace_command".to_string(),
            status: mission_status,
            started_unix_ms: record.command.started_unix_ms,
            completed_unix_ms: Some(completed_unix_ms),
            runtime_ms: Some(evidence.elapsed_ms),
            cost_usd: None,
            summary: Some(command_evidence_summary(&evidence)),
            artifact_ids: vec![stdout_artifact_id, stderr_artifact_id],
            verification_gate_ids,
            metadata: BTreeMap::from([
                ("argv".to_string(), json!(evidence.argv)),
                ("cwd".to_string(), json!(evidence.cwd.display().to_string())),
                (
                    "stdout_path".to_string(),
                    json!(evidence.stdout_path.display().to_string()),
                ),
                (
                    "stderr_path".to_string(),
                    json!(evidence.stderr_path.display().to_string()),
                ),
                ("exit_status".to_string(), json!(evidence.exit_status)),
                ("reason_code".to_string(), json!(evidence.reason_code)),
                ("denied_reason".to_string(), json!(evidence.denied_reason)),
            ]),
        })
        .map_err(CodingMissionError::MissionToolEvidence)?;

    state.updated_unix_ms = completed_unix_ms;
    state.mission.latest_output_summary = command_evidence_summary(&evidence);
    state.mission.latest_verifier = Some(MissionVerifierRecord {
        kind: "coding_workspace_command".to_string(),
        status: match evidence.status {
            CodingWorkspaceCommandStatus::Succeeded => MissionVerifierStatus::Continue,
            CodingWorkspaceCommandStatus::Failed | CodingWorkspaceCommandStatus::Denied => {
                MissionVerifierStatus::Failed
            }
        },
        reason_code: evidence.reason_code.clone(),
        message: command_evidence_summary(&evidence),
        details: BTreeMap::from([
            ("command_id".to_string(), json!(evidence.command_id)),
            ("status".to_string(), json!(evidence.status)),
            ("denied_reason".to_string(), json!(evidence.denied_reason)),
        ]),
    });
    state.events.push(CodingMissionEvent {
        phase: state.phase,
        reason_code: evidence.reason_code.clone(),
        message: command_evidence_summary(&evidence),
        created_unix_ms: completed_unix_ms,
    });
    state.command_evidence.push(evidence.clone());
    save_coding_mission_state(state)?;
    Ok(evidence)
}

fn coding_mission_command_artifact_path(
    state_root: &Path,
    mission_id: &str,
    command_id: &str,
    stream: &str,
) -> PathBuf {
    state_root
        .join("coding-missions")
        .join(mission_id)
        .join("commands")
        .join(format!("{command_id}.{stream}"))
}

fn command_evidence_summary(evidence: &CodingWorkspaceCommandEvidence) -> String {
    match evidence.status {
        CodingWorkspaceCommandStatus::Succeeded => {
            format!("command {} succeeded", evidence.command_id)
        }
        CodingWorkspaceCommandStatus::Failed => format!(
            "command {} failed with status {:?}",
            evidence.command_id, evidence.exit_status
        ),
        CodingWorkspaceCommandStatus::Denied => format!(
            "command {} denied: {}",
            evidence.command_id,
            evidence.denied_reason.as_deref().unwrap_or("policy_denied")
        ),
    }
}

fn matching_verification_gate_ids(state: &CodingMissionState, argv: &[String]) -> Vec<String> {
    let command = argv.join(" ");
    state
        .mission
        .verification_gates
        .iter()
        .filter(|gate| {
            gate.id == format!("verifier:{command}")
                || gate
                    .evidence
                    .get("command")
                    .and_then(|value| value.as_str())
                    .is_some_and(|candidate| candidate == command)
        })
        .map(|gate| gate.id.clone())
        .collect()
}

fn elapsed_ms_saturating(started: Instant) -> u64 {
    let millis = started.elapsed().as_millis();
    if millis > u128::from(u64::MAX) {
        u64::MAX
    } else {
        millis as u64
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
    #[error("failed to record coding mission tool evidence: {0}")]
    MissionToolEvidence(#[from] MissionToolEvidenceError),
    #[error("coding mission git lifecycle blocked ({reason_code}): {message}")]
    GitLifecycle {
        reason_code: &'static str,
        message: String,
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

fn path_is_allowed_by_roots(path: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| path.starts_with(root))
}

fn command_program(argv: &[String]) -> String {
    Path::new(&argv[0])
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(argv[0].as_str())
        .to_ascii_lowercase()
}

fn command_is_force_push(argv: &[String]) -> bool {
    if command_program(argv) != "git" {
        return false;
    }
    argv.get(1).is_some_and(|subcommand| subcommand == "push")
        && argv
            .iter()
            .skip(2)
            .any(|arg| arg == "--force" || arg == "-f" || arg == "--force-with-lease")
}

fn command_is_destructive(argv: &[String]) -> bool {
    let program = command_program(argv);
    if program == "rm" && rm_args_are_recursive_force(argv) {
        return true;
    }
    if program == "git" {
        return argv.get(1).is_some_and(|subcommand| {
            (subcommand == "reset" && argv.iter().skip(2).any(|arg| arg == "--hard"))
                || (subcommand == "clean"
                    && argv
                        .iter()
                        .skip(2)
                        .any(|arg| arg.starts_with('-') && arg.contains('f')))
        });
    }
    false
}

fn rm_args_are_recursive_force(argv: &[String]) -> bool {
    let has_recursive = argv
        .iter()
        .skip(1)
        .any(|arg| arg == "-r" || arg == "-R" || arg == "--recursive" || flag_contains(arg, 'r'));
    let has_force = argv
        .iter()
        .skip(1)
        .any(|arg| arg == "-f" || arg == "--force" || flag_contains(arg, 'f'));
    has_recursive && has_force
}

fn flag_contains(arg: &str, needle: char) -> bool {
    arg.starts_with('-')
        && !arg.starts_with("--")
        && arg.chars().any(|candidate| candidate == needle)
}

fn command_uses_network(argv: &[String]) -> bool {
    let program = command_program(argv);
    if matches!(
        program.as_str(),
        "curl" | "wget" | "ssh" | "scp" | "rsync" | "nc" | "ncat"
    ) {
        return true;
    }
    program == "git"
        && argv.get(1).is_some_and(|subcommand| {
            matches!(
                subcommand.as_str(),
                "clone" | "fetch" | "pull" | "push" | "ls-remote" | "submodule"
            )
        })
}

fn command_requires_mutation(argv: &[String]) -> bool {
    let program = command_program(argv);
    if matches!(
        program.as_str(),
        "touch" | "mkdir" | "rmdir" | "rm" | "mv" | "cp" | "install" | "tee"
    ) {
        return true;
    }
    program == "git"
        && argv.get(1).is_some_and(|subcommand| {
            matches!(
                subcommand.as_str(),
                "add"
                    | "am"
                    | "apply"
                    | "checkout"
                    | "clean"
                    | "commit"
                    | "merge"
                    | "rebase"
                    | "reset"
                    | "restore"
                    | "stash"
                    | "switch"
            )
        })
}

fn command_has_out_of_root_write(cwd: &Path, argv: &[String], roots: &[PathBuf]) -> bool {
    if !command_requires_mutation(argv) {
        return false;
    }
    let program = command_program(argv);
    if program == "git" {
        return false;
    }
    argv.iter()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .filter_map(|arg| lexical_command_path(cwd, arg))
        .any(|candidate| !path_is_allowed_by_roots(&candidate, roots))
}

fn lexical_command_path(cwd: &Path, arg: &str) -> Option<PathBuf> {
    if arg.trim().is_empty() {
        return None;
    }
    let raw = Path::new(arg);
    let absolute = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        cwd.join(raw)
    };
    Some(normalize_path_lexically(&absolute))
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
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

fn write_bytes_atomic(path: &Path, payload: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("coding-mission-artifact");
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
        let init = std::process::Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(&repo)
            .status()
            .expect("git init");
        assert!(init.success());
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

    #[test]
    fn spec_c03_workspace_executor_records_durable_command_evidence() {
        let temp = tempdir().expect("tempdir");
        let mut state = CodingMissionState::create(config_for(temp.path())).expect("create state");
        let repo_path = state.repo_path.clone();
        let executor = CodingWorkspaceExecutor::new(CodingWorkspaceCommandPolicy {
            allowed_roots: state.allowed_roots.clone(),
            allow_network: false,
            allow_mutation: false,
        });

        let status = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path.clone(),
                    argv: vec![
                        "git".to_string(),
                        "status".to_string(),
                        "--short".to_string(),
                    ],
                    reason_code: "git_status".to_string(),
                    started_unix_ms: 1_800_000_000_200,
                },
            )
            .expect("git status evidence");
        let diff = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path.clone(),
                    argv: vec!["git".to_string(), "diff".to_string(), "--stat".to_string()],
                    reason_code: "git_diff".to_string(),
                    started_unix_ms: 1_800_000_000_300,
                },
            )
            .expect("git diff evidence");
        let verifier = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path.clone(),
                    argv: vec!["git".to_string(), "--version".to_string()],
                    reason_code: "verifier_git_version".to_string(),
                    started_unix_ms: 1_800_000_000_400,
                },
            )
            .expect("verifier evidence");

        assert_eq!(status.status, CodingWorkspaceCommandStatus::Succeeded);
        assert_eq!(diff.status, CodingWorkspaceCommandStatus::Succeeded);
        assert_eq!(verifier.status, CodingWorkspaceCommandStatus::Succeeded);
        assert_eq!(state.command_evidence.len(), 3);
        assert_eq!(state.mission.tool_evidence.len(), 3);
        assert_eq!(state.command_evidence[0].reason_code, "git_status");
        assert_eq!(
            state.command_evidence[0].argv,
            vec!["git", "status", "--short"]
        );
        assert!(state.command_evidence[0].stdout_path.is_file());
        assert!(state.command_evidence[0].stderr_path.is_file());
        assert_eq!(state.command_evidence[0].exit_status, Some(0));
        assert!(std::fs::read_to_string(&verifier.stdout_path)
            .expect("verifier stdout")
            .contains("git version"));

        let loaded =
            load_coding_mission_state(state.state_root.as_path(), state.mission_id.as_str())
                .expect("load persisted state");
        assert_eq!(loaded.command_evidence, state.command_evidence);
        assert_eq!(loaded.mission.tool_evidence.len(), 3);
        assert!(loaded
            .events
            .iter()
            .any(|event| event.reason_code == "workspace_command_started"));
        assert!(loaded
            .mission
            .artifacts
            .iter()
            .any(|artifact| artifact.kind == "workspace_command_started"));
    }

    #[test]
    fn regression_workspace_executor_denies_destructive_command_without_mutation() {
        let temp = tempdir().expect("tempdir");
        let mut state = CodingMissionState::create(config_for(temp.path())).expect("create state");
        let repo_path = state.repo_path.clone();
        let keep = repo_path.join("keep.txt");
        std::fs::write(&keep, "do not delete").expect("seed file");
        let executor = CodingWorkspaceExecutor::new(CodingWorkspaceCommandPolicy {
            allowed_roots: state.allowed_roots.clone(),
            allow_network: false,
            allow_mutation: true,
        });

        let denied = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path,
                    argv: vec!["rm".to_string(), "-rf".to_string(), "keep.txt".to_string()],
                    reason_code: "rm_rf".to_string(),
                    started_unix_ms: 1_800_000_000_500,
                },
            )
            .expect("denied evidence");

        assert_eq!(denied.status, CodingWorkspaceCommandStatus::Denied);
        assert_eq!(
            denied.denied_reason.as_deref(),
            Some("denied_destructive_command")
        );
        assert!(keep.is_file());
        assert!(denied.stdout_path.is_file());
        assert!(denied.stderr_path.is_file());
    }

    #[test]
    fn regression_workspace_executor_denies_force_push_network_and_missing_cwd() {
        let temp = tempdir().expect("tempdir");
        let mut state = CodingMissionState::create(config_for(temp.path())).expect("create state");
        let repo_path = state.repo_path.clone();
        let executor = CodingWorkspaceExecutor::new(CodingWorkspaceCommandPolicy {
            allowed_roots: state.allowed_roots.clone(),
            allow_network: false,
            allow_mutation: true,
        });

        let force_push = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path.clone(),
                    argv: vec![
                        "git".to_string(),
                        "push".to_string(),
                        "--force".to_string(),
                        "origin".to_string(),
                        "main".to_string(),
                    ],
                    reason_code: "force_push".to_string(),
                    started_unix_ms: 1_800_000_000_600,
                },
            )
            .expect("force push denial");
        let network = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path,
                    argv: vec!["curl".to_string(), "https://example.com".to_string()],
                    reason_code: "network_probe".to_string(),
                    started_unix_ms: 1_800_000_000_700,
                },
            )
            .expect("network denial");
        let missing = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: temp.path().join("missing"),
                    argv: vec!["git".to_string(), "status".to_string()],
                    reason_code: "missing_cwd".to_string(),
                    started_unix_ms: 1_800_000_000_800,
                },
            )
            .expect("missing cwd denial");

        assert_eq!(
            force_push.denied_reason.as_deref(),
            Some("denied_force_push")
        );
        assert_eq!(network.denied_reason.as_deref(), Some("network_denied"));
        assert_eq!(missing.denied_reason.as_deref(), Some("cwd_missing"));
        assert_eq!(state.command_evidence.len(), 3);
        assert!(state
            .command_evidence
            .iter()
            .all(|evidence| evidence.status == CodingWorkspaceCommandStatus::Denied));
    }

    #[test]
    fn regression_workspace_executor_denies_out_of_root_write() {
        let temp = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let outside_file = outside.path().join("outside.txt");
        let mut state = CodingMissionState::create(config_for(temp.path())).expect("create state");
        let repo_path = state.repo_path.clone();
        let executor = CodingWorkspaceExecutor::new(CodingWorkspaceCommandPolicy {
            allowed_roots: state.allowed_roots.clone(),
            allow_network: false,
            allow_mutation: true,
        });

        let denied = executor
            .execute(
                &mut state,
                CodingWorkspaceCommand {
                    cwd: repo_path,
                    argv: vec!["touch".to_string(), outside_file.display().to_string()],
                    reason_code: "out_of_root_touch".to_string(),
                    started_unix_ms: 1_800_000_000_900,
                },
            )
            .expect("out of root denial");

        assert_eq!(denied.status, CodingWorkspaceCommandStatus::Denied);
        assert_eq!(denied.denied_reason.as_deref(), Some("out_of_root_write"));
        assert!(!outside_file.exists());
    }

    fn git(repo: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .expect("git command");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn committed_config_for(root: &Path) -> CodingMissionConfig {
        let mut config = config_for(root);
        git(
            &config.repo_path,
            &["config", "user.email", "tau@example.test"],
        );
        git(&config.repo_path, &["config", "user.name", "Tau Test"]);
        std::fs::write(config.repo_path.join("README.md"), "initial\n").expect("seed readme");
        git(&config.repo_path, &["add", "README.md"]);
        git(&config.repo_path, &["commit", "-m", "initial"]);
        config.base_branch = git(&config.repo_path, &["branch", "--show-current"]);
        config.branch_prefix = "codex/".to_string();
        config
    }

    #[test]
    fn spec_c04_prepare_branch_creates_and_reuses_local_branch() {
        let temp = tempdir().expect("tempdir");
        let config = committed_config_for(temp.path());
        let mut state = CodingMissionState::create(config).expect("create state");

        let created = state
            .prepare_branch(CodingGitPrepareBranchRequest {
                branch_name: None,
                allow_fetch: false,
                started_unix_ms: 1_800_000_001_000,
            })
            .expect("create branch");
        let reused = state
            .prepare_branch(CodingGitPrepareBranchRequest {
                branch_name: None,
                allow_fetch: false,
                started_unix_ms: 1_800_000_001_100,
            })
            .expect("reuse branch");

        assert_eq!(created.kind, CodingGitLifecycleEvidenceKind::BranchPrepared);
        assert_eq!(created.branch_name, "codex/coding-mission-alpha");
        assert!(created.created_branch);
        assert!(!created.reused_branch);
        assert_eq!(reused.branch_name, created.branch_name);
        assert!(!reused.created_branch);
        assert!(reused.reused_branch);
        assert_eq!(
            git(&state.repo_path, &["branch", "--show-current"]),
            "codex/coding-mission-alpha"
        );
        assert_eq!(state.git_evidence.len(), 2);
        assert!(state
            .command_evidence
            .iter()
            .any(|evidence| evidence.reason_code == "git_switch_create_branch"));
    }

    #[test]
    fn regression_prepare_branch_blocks_dirty_tree_before_checkout() {
        let temp = tempdir().expect("tempdir");
        let config = committed_config_for(temp.path());
        let mut state = CodingMissionState::create(config).expect("create state");
        std::fs::write(state.repo_path.join("dirty.txt"), "user work\n").expect("dirty file");

        let error = state
            .prepare_branch(CodingGitPrepareBranchRequest {
                branch_name: None,
                allow_fetch: false,
                started_unix_ms: 1_800_000_001_200,
            })
            .expect_err("dirty tree should block");

        assert!(matches!(
            error,
            CodingMissionError::GitLifecycle {
                reason_code: "dirty_tree",
                ..
            }
        ));
        assert_eq!(
            git(&state.repo_path, &["branch", "--show-current"]),
            state.base_branch
        );
        assert!(state.repo_path.join("dirty.txt").is_file());
        assert!(state.git_evidence.is_empty());
    }

    #[test]
    fn regression_prepare_branch_blocks_base_branch_mismatch() {
        let temp = tempdir().expect("tempdir");
        let config = committed_config_for(temp.path());
        let mut state = CodingMissionState::create(config).expect("create state");
        git(&state.repo_path, &["switch", "-c", "other-work"]);

        let error = state
            .prepare_branch(CodingGitPrepareBranchRequest {
                branch_name: None,
                allow_fetch: false,
                started_unix_ms: 1_800_000_001_300,
            })
            .expect_err("wrong base should block");

        assert!(matches!(
            error,
            CodingMissionError::GitLifecycle {
                reason_code: "base_branch_mismatch",
                ..
            }
        ));
        assert_eq!(
            git(&state.repo_path, &["branch", "--show-current"]),
            "other-work"
        );
    }

    #[test]
    fn spec_c05_commit_changes_requires_diff_and_records_hash() {
        let temp = tempdir().expect("tempdir");
        let config = committed_config_for(temp.path());
        let mut state = CodingMissionState::create(config).expect("create state");
        state
            .prepare_branch(CodingGitPrepareBranchRequest {
                branch_name: None,
                allow_fetch: false,
                started_unix_ms: 1_800_000_001_400,
            })
            .expect("prepare branch");
        std::fs::write(state.repo_path.join("mission.txt"), "mission change\n")
            .expect("mission file");

        let commit = state
            .commit_changes(CodingGitCommitRequest {
                message: "Implement mission change".to_string(),
                started_unix_ms: 1_800_000_001_500,
            })
            .expect("commit changes");

        assert_eq!(commit.kind, CodingGitLifecycleEvidenceKind::CommitCreated);
        assert_eq!(commit.branch_name, "codex/coding-mission-alpha");
        assert_eq!(commit.changed_files, vec!["mission.txt"]);
        assert_eq!(commit.commit_hash.as_deref().map(str::len), Some(40));
        assert!(git(&state.repo_path, &["log", "-1", "--format=%B"])
            .contains("Mission: coding-mission-alpha"));
        assert!(state
            .git_evidence
            .iter()
            .any(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated));
    }

    #[test]
    fn regression_commit_changes_requires_non_empty_diff() {
        let temp = tempdir().expect("tempdir");
        let config = committed_config_for(temp.path());
        let mut state = CodingMissionState::create(config).expect("create state");
        state
            .prepare_branch(CodingGitPrepareBranchRequest {
                branch_name: None,
                allow_fetch: false,
                started_unix_ms: 1_800_000_001_600,
            })
            .expect("prepare branch");

        let error = state
            .commit_changes(CodingGitCommitRequest {
                message: "No diff".to_string(),
                started_unix_ms: 1_800_000_001_700,
            })
            .expect_err("empty diff should block");

        assert!(matches!(
            error,
            CodingMissionError::GitLifecycle {
                reason_code: "missing_git_diff",
                ..
            }
        ));
    }

    fn verifier_config_for(
        root: &Path,
        initial_status: &str,
        verifier: &str,
    ) -> CodingMissionConfig {
        let mut config = committed_config_for(root);
        std::fs::write(config.repo_path.join("status.txt"), initial_status).expect("status file");
        git(&config.repo_path, &["add", "status.txt"]);
        git(&config.repo_path, &["commit", "-m", "add status fixture"]);
        config.verifier_commands = vec![verifier.to_string()];
        config
    }

    fn multi_file_verifier_config_for(root: &Path) -> CodingMissionConfig {
        let mut config = committed_config_for(root);
        std::fs::write(config.repo_path.join("status.txt"), "fail\n").expect("status file");
        std::fs::create_dir_all(config.repo_path.join("docs")).expect("docs dir");
        std::fs::write(config.repo_path.join("docs/notes.txt"), "missing\n").expect("notes file");
        git(&config.repo_path, &["add", "status.txt", "docs/notes.txt"]);
        git(
            &config.repo_path,
            &["commit", "-m", "add multi file fixture"],
        );
        config.verifier_commands = vec![
            "grep -q pass status.txt".to_string(),
            "grep -q helper docs/notes.txt".to_string(),
        ];
        config
    }

    #[test]
    fn spec_c06_outer_loop_records_red_and_stays_executing() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "fail\n", "grep -q pass status.txt");
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "Make verifier green".to_string(),
                    started_unix_ms: 1_800_000_002_000,
                },
            )
            .expect("run mission");

        assert_eq!(outcome.phase, CodingMissionPhase::Executing);
        assert!(!outcome.verifier_passed);
        assert!(outcome.committed.is_none());
        assert!(outcome.blocked_reason.is_none());
        assert_eq!(state.phase, CodingMissionPhase::Executing);
        assert!(state.command_evidence.iter().any(|evidence| evidence.status
            == CodingWorkspaceCommandStatus::Failed
            && evidence.reason_code.starts_with("coding_verifier")));
        assert!(!state
            .git_evidence
            .iter()
            .any(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated));
    }

    #[test]
    fn spec_c07_outer_loop_edits_verifies_commits_and_becomes_pr_ready() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "fail\n", "grep -q pass status.txt");
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: Some(CodingMissionControlledEdit {
                        relative_path: PathBuf::from("status.txt"),
                        contents: "pass\n".to_string(),
                        reason_code: "controlled_fix".to_string(),
                    }),
                    controlled_edits: Vec::new(),
                    commit_message: "Make verifier green".to_string(),
                    started_unix_ms: 1_800_000_002_500,
                },
            )
            .expect("run mission");

        assert_eq!(outcome.phase, CodingMissionPhase::PrReady);
        assert!(outcome.verifier_passed);
        assert_eq!(
            outcome
                .committed
                .as_ref()
                .and_then(|e| e.commit_hash.as_ref())
                .map(String::len),
            Some(40)
        );
        assert_eq!(state.phase, CodingMissionPhase::PrReady);
        assert!(state.command_evidence.iter().any(|evidence| evidence.status
            == CodingWorkspaceCommandStatus::Failed
            && evidence.reason_code.starts_with("coding_verifier")));
        assert!(state.command_evidence.iter().any(|evidence| evidence.status
            == CodingWorkspaceCommandStatus::Succeeded
            && evidence.reason_code.starts_with("coding_verifier")));
        assert!(git(&state.repo_path, &["log", "-1", "--format=%B"])
            .contains("Mission: coding-mission-alpha"));
    }

    #[test]
    fn spec_c12_outer_loop_applies_multi_file_edits_and_commits() {
        let temp = tempdir().expect("tempdir");
        let config = multi_file_verifier_config_for(temp.path());
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: vec![
                        CodingMissionControlledEdit {
                            relative_path: PathBuf::from("status.txt"),
                            contents: "pass\n".to_string(),
                            reason_code: "controlled_fix_status".to_string(),
                        },
                        CodingMissionControlledEdit {
                            relative_path: PathBuf::from("docs/notes.txt"),
                            contents: "helper\n".to_string(),
                            reason_code: "controlled_fix_notes".to_string(),
                        },
                    ],
                    commit_message: "Make multi-file verifier green".to_string(),
                    started_unix_ms: 1_800_000_002_700,
                },
            )
            .expect("run multi-file mission");

        assert_eq!(outcome.phase, CodingMissionPhase::PrReady);
        assert!(outcome.verifier_passed);
        assert_eq!(
            outcome
                .committed
                .as_ref()
                .map(|commit| commit.changed_files.clone()),
            Some(vec!["docs/notes.txt".to_string(), "status.txt".to_string()])
        );
        assert_eq!(
            std::fs::read_to_string(state.repo_path.join("status.txt")).expect("status"),
            "pass\n"
        );
        assert_eq!(
            std::fs::read_to_string(state.repo_path.join("docs/notes.txt")).expect("notes"),
            "helper\n"
        );
        assert!(state
            .mission
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_id == "controlled_edit:status.txt"));
        assert!(state
            .mission
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_id == "controlled_edit:docs/notes.txt"));
    }

    #[test]
    fn regression_outer_loop_blocks_multi_file_escape_before_partial_write() {
        let temp = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        let mut config = multi_file_verifier_config_for(temp.path());
        config.allowed_roots.push(outside.path().to_path_buf());
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: vec![
                        CodingMissionControlledEdit {
                            relative_path: PathBuf::from("status.txt"),
                            contents: "pass\n".to_string(),
                            reason_code: "controlled_fix_status".to_string(),
                        },
                        CodingMissionControlledEdit {
                            relative_path: PathBuf::from("../escape.txt"),
                            contents: "escape\n".to_string(),
                            reason_code: "controlled_escape".to_string(),
                        },
                    ],
                    commit_message: "Reject escaped edit".to_string(),
                    started_unix_ms: 1_800_000_002_800,
                },
            )
            .expect("run escaped multi-file mission");

        assert_eq!(outcome.phase, CodingMissionPhase::Blocked);
        assert_eq!(
            outcome.blocked_reason.as_deref(),
            Some("controlled_edit_outside_repo")
        );
        assert_eq!(
            std::fs::read_to_string(state.repo_path.join("status.txt")).expect("status"),
            "fail\n"
        );
        assert!(!outside.path().join("escape.txt").exists());
        assert!(!state
            .git_evidence
            .iter()
            .any(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated));
    }

    #[test]
    fn regression_outer_loop_blocks_completion_without_mutation_evidence() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "pass\n", "grep -q pass status.txt");
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "No mutation".to_string(),
                    started_unix_ms: 1_800_000_003_000,
                },
            )
            .expect("run mission");

        assert_eq!(outcome.phase, CodingMissionPhase::Blocked);
        assert!(outcome.verifier_passed);
        assert_eq!(outcome.blocked_reason.as_deref(), Some("missing_git_diff"));
        assert_eq!(state.phase, CodingMissionPhase::Blocked);
    }

    #[test]
    fn regression_outer_loop_blocks_impossible_verifier_command() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "fail\n", "definitely-not-a-tau-command");
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "Impossible verifier".to_string(),
                    started_unix_ms: 1_800_000_003_500,
                },
            )
            .expect("run mission");

        assert_eq!(outcome.phase, CodingMissionPhase::Blocked);
        assert_eq!(
            outcome.blocked_reason.as_deref(),
            Some("verifier_command_failed_to_start")
        );
        assert!(state
            .command_evidence
            .iter()
            .any(|evidence| evidence.exit_status.is_none()
                && evidence.status == CodingWorkspaceCommandStatus::Failed));
    }

    #[test]
    fn regression_outer_loop_blocks_policy_denied_verifier_command() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "fail\n", "rm -rf status.txt");
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "Denied verifier".to_string(),
                    started_unix_ms: 1_800_000_004_000,
                },
            )
            .expect("run mission");

        assert_eq!(outcome.phase, CodingMissionPhase::Blocked);
        assert_eq!(
            outcome.blocked_reason.as_deref(),
            Some("verifier_command_denied")
        );
        assert!(state.repo_path.join("status.txt").is_file());
    }

    #[test]
    fn regression_outer_loop_blocks_missing_verifier_commands() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "pass\n", "grep -q pass status.txt");
        let mut state = CodingMissionState::create(config).expect("create state");
        state.verifier_commands.clear();
        let runner = CodingMissionRunner::new();

        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: Some(CodingMissionControlledEdit {
                        relative_path: PathBuf::from("status.txt"),
                        contents: "pass again\n".to_string(),
                        reason_code: "controlled_fix".to_string(),
                    }),
                    controlled_edits: Vec::new(),
                    commit_message: "No verifier".to_string(),
                    started_unix_ms: 1_800_000_004_500,
                },
            )
            .expect("run mission");

        assert_eq!(outcome.phase, CodingMissionPhase::Blocked);
        assert!(!outcome.verifier_passed);
        assert_eq!(
            outcome.blocked_reason.as_deref(),
            Some("missing_verifier_commands")
        );
        assert!(!state
            .git_evidence
            .iter()
            .any(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated));
    }

    #[test]
    fn spec_c08_resume_after_red_checkpoint_restores_branch_and_completes() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "fail\n", "grep -q pass status.txt");
        let state_root = config.state_root.clone();
        let mission_id = config.mission_id.clone();
        let repo_path = config.repo_path.clone();
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        let first = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "Make verifier green".to_string(),
                    started_unix_ms: 1_800_000_005_000,
                },
            )
            .expect("run red mission");

        assert_eq!(first.phase, CodingMissionPhase::Executing);
        assert_eq!(
            state
                .resume_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.next_action),
            Some(CodingMissionResumeAction::ApplyEdit)
        );
        assert_eq!(
            state
                .resume_checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.branch_name.as_deref()),
            Some("codex/coding-mission-alpha")
        );
        assert!(state.command_evidence.iter().any(|evidence| evidence.status
            == CodingWorkspaceCommandStatus::Failed
            && evidence.reason_code.starts_with("coding_verifier")));

        drop(state);
        git(&repo_path, &["switch", "master"]);

        let resumed = runner
            .resume(CodingMissionResumeRequest {
                state_root: state_root.clone(),
                mission_id: mission_id.clone(),
                controlled_edit: Some(CodingMissionControlledEdit {
                    relative_path: PathBuf::from("status.txt"),
                    contents: "pass\n".to_string(),
                    reason_code: "controlled_fix_after_red_resume".to_string(),
                }),
                controlled_edits: Vec::new(),
                commit_message: "Make verifier green after resume".to_string(),
                started_unix_ms: 1_800_000_005_500,
                stop_after: None,
            })
            .expect("resume mission");

        assert_eq!(resumed.resume_action, CodingMissionResumeAction::ApplyEdit);
        assert_eq!(
            resumed.restored_branch.as_deref(),
            Some("codex/coding-mission-alpha")
        );
        assert_eq!(resumed.run.phase, CodingMissionPhase::PrReady);
        assert_eq!(resumed.state.phase, CodingMissionPhase::PrReady);
        assert_eq!(
            git(&resumed.state.repo_path, &["branch", "--show-current"]),
            "codex/coding-mission-alpha"
        );
        assert!(resumed
            .state
            .command_evidence
            .iter()
            .any(
                |evidence| evidence.status == CodingWorkspaceCommandStatus::Failed
                    && evidence.reason_code.starts_with("coding_verifier")
            ));
        assert!(resumed
            .state
            .command_evidence
            .iter()
            .any(
                |evidence| evidence.status == CodingWorkspaceCommandStatus::Succeeded
                    && evidence.reason_code.starts_with("coding_verifier")
            ));
        assert!(resumed
            .state
            .mission
            .latest_output_summary
            .contains("verifier passed"));
    }

    #[test]
    fn spec_c09_resume_after_edit_blocks_on_mutation_fingerprint_drift() {
        let temp = tempdir().expect("tempdir");
        let config = verifier_config_for(temp.path(), "fail\n", "grep -q pass status.txt");
        let state_root = config.state_root.clone();
        let mission_id = config.mission_id.clone();
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "Make verifier green".to_string(),
                    started_unix_ms: 1_800_000_006_000,
                },
            )
            .expect("run red mission");

        let paused = runner
            .resume(CodingMissionResumeRequest {
                state_root: state_root.clone(),
                mission_id: mission_id.clone(),
                controlled_edit: Some(CodingMissionControlledEdit {
                    relative_path: PathBuf::from("status.txt"),
                    contents: "pass\n".to_string(),
                    reason_code: "controlled_fix_before_crash".to_string(),
                }),
                controlled_edits: Vec::new(),
                commit_message: "Make verifier green after edit".to_string(),
                started_unix_ms: 1_800_000_006_500,
                stop_after: Some(CodingMissionResumeStopAfter::ControlledEdit),
            })
            .expect("resume to edit checkpoint");

        assert_eq!(paused.run.phase, CodingMissionPhase::Executing);
        assert_eq!(
            paused
                .state
                .resume_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.next_action),
            Some(CodingMissionResumeAction::VerifyAfterEdit)
        );
        assert!(paused
            .state
            .resume_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.mutation_fingerprint.as_ref())
            .is_some());

        std::fs::write(
            paused.state.repo_path.join("status.txt"),
            "pass\nuser drift\n",
        )
        .expect("drift status");

        let blocked = runner
            .resume(CodingMissionResumeRequest {
                state_root,
                mission_id,
                controlled_edit: None,
                controlled_edits: Vec::new(),
                commit_message: "Commit resumed edit".to_string(),
                started_unix_ms: 1_800_000_007_000,
                stop_after: None,
            })
            .expect("resume after drift");

        assert_eq!(blocked.run.phase, CodingMissionPhase::Blocked);
        assert_eq!(
            blocked.run.blocked_reason.as_deref(),
            Some("mutation_fingerprint_mismatch")
        );
        assert!(
            std::fs::read_to_string(blocked.state.repo_path.join("status.txt"))
                .expect("read status")
                .contains("user drift")
        );
        assert!(!blocked
            .state
            .git_evidence
            .iter()
            .any(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated));
    }

    #[test]
    fn spec_c13_resume_after_multi_file_edit_completes_with_full_fingerprint() {
        let temp = tempdir().expect("tempdir");
        let config = multi_file_verifier_config_for(temp.path());
        let state_root = config.state_root.clone();
        let mission_id = config.mission_id.clone();
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();

        runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: None,
                    controlled_edits: Vec::new(),
                    commit_message: "Prepare multi-file resume".to_string(),
                    started_unix_ms: 1_800_000_007_500,
                },
            )
            .expect("run red mission");

        let paused = runner
            .resume(CodingMissionResumeRequest {
                state_root: state_root.clone(),
                mission_id: mission_id.clone(),
                controlled_edit: None,
                controlled_edits: vec![
                    CodingMissionControlledEdit {
                        relative_path: PathBuf::from("status.txt"),
                        contents: "pass\n".to_string(),
                        reason_code: "controlled_fix_status_after_resume".to_string(),
                    },
                    CodingMissionControlledEdit {
                        relative_path: PathBuf::from("docs/notes.txt"),
                        contents: "helper\n".to_string(),
                        reason_code: "controlled_fix_notes_after_resume".to_string(),
                    },
                ],
                commit_message: "Apply multi-file resume edit".to_string(),
                started_unix_ms: 1_800_000_008_000,
                stop_after: Some(CodingMissionResumeStopAfter::ControlledEdit),
            })
            .expect("resume through multi-file edit");

        let fingerprint = paused
            .state
            .resume_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.mutation_fingerprint.as_ref())
            .expect("multi-file fingerprint");
        assert_eq!(
            fingerprint.changed_files,
            vec!["docs/notes.txt".to_string(), "status.txt".to_string()]
        );

        let resumed = runner
            .resume(CodingMissionResumeRequest {
                state_root,
                mission_id,
                controlled_edit: None,
                controlled_edits: Vec::new(),
                commit_message: "Commit multi-file resumed edit".to_string(),
                started_unix_ms: 1_800_000_008_500,
                stop_after: None,
            })
            .expect("resume after multi-file edit");

        assert_eq!(resumed.run.phase, CodingMissionPhase::PrReady);
        assert_eq!(
            resumed
                .run
                .committed
                .as_ref()
                .map(|commit| commit.changed_files.clone()),
            Some(vec!["docs/notes.txt".to_string(), "status.txt".to_string()])
        );
    }

    fn pr_ready_state_for(root: &Path) -> (CodingMissionState, CodingGitLifecycleEvidence) {
        let config = verifier_config_for(root, "fail\n", "grep -q pass status.txt");
        let mut state = CodingMissionState::create(config).expect("create state");
        let runner = CodingMissionRunner::new();
        let outcome = runner
            .run(
                &mut state,
                CodingMissionRunRequest {
                    controlled_edit: Some(CodingMissionControlledEdit {
                        relative_path: PathBuf::from("status.txt"),
                        contents: "pass\n".to_string(),
                        reason_code: "controlled_fix_for_pr_ready".to_string(),
                    }),
                    controlled_edits: Vec::new(),
                    commit_message: "Make verifier green".to_string(),
                    started_unix_ms: 1_800_000_007_500,
                },
            )
            .expect("run mission to pr_ready");
        assert_eq!(outcome.phase, CodingMissionPhase::PrReady);
        (state, outcome.committed.expect("commit evidence"))
    }

    #[test]
    fn spec_c10_pr_ready_bundle_without_auth_records_manual_gh_command() {
        let temp = tempdir().expect("tempdir");
        let (mut state, commit) = pr_ready_state_for(temp.path());

        let bundle = state
            .prepare_pr_ready_bundle(CodingMissionPrReadyRequest {
                title: Some("Make verifier green".to_string()),
                risk_notes: vec!["Risk: verifier scope is local".to_string()],
                rollback_notes: vec!["Rollback: revert the mission commit".to_string()],
                allow_draft_pr: true,
                github_env: BTreeMap::new(),
                gh_binary: None,
                started_unix_ms: 1_800_000_008_000,
            })
            .expect("prepare pr-ready bundle");

        assert_eq!(bundle.status, CodingMissionPrPublicationStatus::ManualReady);
        assert_eq!(bundle.branch_name, "codex/coding-mission-alpha");
        assert_eq!(bundle.commit_hash.as_deref(), commit.commit_hash.as_deref());
        assert_eq!(bundle.title, "Make verifier green");
        assert!(bundle.body.contains("Mission: coding-mission-alpha"));
        assert!(bundle.body.contains("Changed files:\n- status.txt"));
        assert!(bundle.body.contains("Verifier evidence:"));
        assert!(bundle.body.contains("Risk: verifier scope is local"));
        assert!(bundle.body.contains("Rollback: revert the mission commit"));
        assert_eq!(bundle.pr_url, None);
        assert!(bundle.manual_gh_pr_create_command.contains(
            "gh pr create --draft --head 'codex/coding-mission-alpha' --title 'Make verifier green'"
        ));
        assert_eq!(
            state
                .pr_ready_bundle
                .as_ref()
                .map(|persisted| persisted.manual_gh_pr_create_command.as_str()),
            Some(bundle.manual_gh_pr_create_command.as_str())
        );
    }

    #[test]
    fn spec_c11_pr_ready_bundle_creates_draft_when_auth_env_present() {
        let temp = tempdir().expect("tempdir");
        let (mut state, _commit) = pr_ready_state_for(temp.path());
        let fake_gh = temp.path().join("fake-gh");
        std::fs::write(
            &fake_gh,
            "#!/bin/sh\nprintf 'https://github.com/njfio/Tau/pull/999\\n'\n",
        )
        .expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake_gh)
                .expect("fake gh metadata")
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake_gh, perms).expect("chmod fake gh");
        }

        let bundle = state
            .prepare_pr_ready_bundle(CodingMissionPrReadyRequest {
                title: Some("Make verifier green".to_string()),
                risk_notes: vec!["Risk: draft PR uses fake gh in test".to_string()],
                rollback_notes: vec!["Rollback: close the draft PR".to_string()],
                allow_draft_pr: true,
                github_env: BTreeMap::from([("GH_TOKEN".to_string(), "test-token".to_string())]),
                gh_binary: Some(fake_gh),
                started_unix_ms: 1_800_000_008_500,
            })
            .expect("prepare draft pr");

        assert_eq!(
            bundle.status,
            CodingMissionPrPublicationStatus::DraftCreated
        );
        assert_eq!(
            bundle.pr_url.as_deref(),
            Some("https://github.com/njfio/Tau/pull/999")
        );
        assert_eq!(
            state
                .pr_ready_bundle
                .as_ref()
                .and_then(|persisted| persisted.pr_url.as_deref()),
            Some("https://github.com/njfio/Tau/pull/999")
        );
        assert!(state.command_evidence.iter().any(|evidence| {
            evidence.reason_code == "gh_pr_create_draft"
                && evidence.status == CodingWorkspaceCommandStatus::Succeeded
        }));
    }
}
