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
            reason_code: reason_code.to_string(),
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
}
