//! Durable autonomous coding job runtime.
//!
//! This layer links `CodingMissionRunner` with the generic background jobs
//! runtime so coding work can be submitted, resumed, recovered, and inspected
//! as one product loop.

use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tau_agent_core::{
    coding_mission_state_path, load_coding_mission_state, save_coding_mission_state,
    CodingGitLifecycleEvidenceKind, CodingMissionConfig, CodingMissionControlledEdit,
    CodingMissionPhase, CodingMissionPrMode, CodingMissionPrPublicationStatus,
    CodingMissionPrReadyBundle, CodingMissionPrReadyRequest, CodingMissionResumeRequest,
    CodingMissionRunRequest, CodingMissionRunner, CodingMissionState, CodingWorkspaceCommandStatus,
};

use crate::{
    run_autonomous_coding_provider_repair, AutonomousCodingProviderRepairEvidence,
    AutonomousCodingProviderRepairPolicy, AutonomousCodingProviderRepairStatus,
    BackgroundJobCreateRequest, BackgroundJobRecord, BackgroundJobRecoveryReport,
    BackgroundJobRuntime, BackgroundJobRuntimeConfig, BackgroundJobTraceContext,
};

mod verifier_derivation;

use verifier_derivation::{
    derive_concrete_docs_verifier_commands, derive_concrete_verifier_commands,
    derive_repo_aware_code_verifier_plan, REPO_AWARE_CARGO_METADATA_REQUIRED_INPUT,
    REPO_AWARE_CARGO_PACKAGE_REQUIRED_PREFIX, REPO_AWARE_TEST_FILTER_REQUIRED_INPUT,
};

pub const AUTONOMOUS_CODING_JOB_SCHEMA_VERSION: u32 = 1;

const AUTONOMOUS_CODING_JOB_REASON_QUEUED: &str = "autonomous_coding_job_queued";
const AUTONOMOUS_CODING_JOB_REASON_RUNNING: &str = "autonomous_coding_job_running";
const AUTONOMOUS_CODING_JOB_REASON_WAITING_FOR_EDIT: &str =
    "autonomous_coding_job_waiting_for_controlled_edit";
const AUTONOMOUS_CODING_JOB_REASON_PR_READY: &str = "autonomous_coding_job_pr_ready";
const AUTONOMOUS_CODING_JOB_REASON_BLOCKED: &str = "autonomous_coding_job_blocked";
const AUTONOMOUS_CODING_JOB_REASON_FAILED: &str = "autonomous_coding_job_failed";
const AUTONOMOUS_CODING_JOB_REASON_RECOVERED: &str = "autonomous_coding_job_background_recovered";
const AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR: &str = "autonomous_coding_job_provider_repair";
const AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR_EXHAUSTED: &str =
    "autonomous_coding_job_provider_repair_exhausted";
const AUTONOMOUS_CODING_JOB_MAX_REPLAY_PASSES: usize = 6;
const AUTONOMOUS_CODING_JOB_LEASE_MS: u64 = 900_000;

static NEXT_AUTONOMOUS_CODING_JOB_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingJobStatus {
    Queued,
    Running,
    Recovering,
    PrReady,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutonomousCodingJobRuntimeConfig {
    pub state_dir: PathBuf,
    pub background_jobs_state_dir: PathBuf,
    pub runner_command: PathBuf,
    pub runner_args_prefix: Vec<String>,
    pub default_timeout_ms: u64,
    pub max_timeout_ms: u64,
    pub worker_poll_ms: u64,
    pub stuck_recovery_poll_ms: u64,
}

impl Default for AutonomousCodingJobRuntimeConfig {
    fn default() -> Self {
        Self {
            state_dir: PathBuf::from(".tau/autonomous-coding"),
            background_jobs_state_dir: PathBuf::from(".tau/jobs"),
            runner_command: PathBuf::from("tau-autonomous-coding-job"),
            runner_args_prefix: Vec::new(),
            default_timeout_ms: 900_000,
            max_timeout_ms: 3_600_000,
            worker_poll_ms: 50,
            stuck_recovery_poll_ms: 250,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobSubmitRequest {
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
    #[serde(default)]
    pub controlled_edits: Vec<CodingMissionControlledEdit>,
    #[serde(default)]
    pub provider_repair: AutonomousCodingProviderRepairPolicy,
    pub commit_message: String,
    pub enqueue_background_job: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobReplayRequest {
    pub job_id: String,
    #[serde(default)]
    pub controlled_edits: Vec<CodingMissionControlledEdit>,
    #[serde(default)]
    pub commit_message: Option<String>,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingMergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl AutonomousCodingMergeMethod {
    fn gh_flag(self) -> &'static str {
        match self {
            Self::Merge => "--merge",
            Self::Squash => "--squash",
            Self::Rebase => "--rebase",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingAutoMergeStatus {
    Requested,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingAutoMergeRequest {
    pub job_id: String,
    pub allow_auto_merge: bool,
    pub merge_method: AutonomousCodingMergeMethod,
    pub delete_branch: bool,
    #[serde(default)]
    pub github_env: BTreeMap<String, String>,
    #[serde(default)]
    pub gh_binary: Option<PathBuf>,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingAutoMergeEvidence {
    pub status: AutonomousCodingAutoMergeStatus,
    pub reason_code: String,
    pub pr_url: Option<String>,
    pub command_argv: Vec<String>,
    #[serde(default)]
    pub stdout_path: Option<PathBuf>,
    #[serde(default)]
    pub stderr_path: Option<PathBuf>,
    #[serde(default)]
    pub exit_status: Option<i32>,
    #[serde(default)]
    pub error_summary: Option<String>,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingAutoMergeOutcome {
    pub record: AutonomousCodingJobRecord,
    pub status: AutonomousCodingJobStatusSnapshot,
    pub evidence: AutonomousCodingAutoMergeEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingIssueIntakeRequest {
    pub intake_id: String,
    pub issue_url: String,
    pub issue_title: String,
    pub issue_body: String,
    pub repo_path: PathBuf,
    pub base_branch: String,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingIssueIntakeStatus {
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingIssueIntakeClassification {
    Ready,
    Solvable,
    Underspecified,
    Unsafe,
    TooBroad,
    #[default]
    MissingVerifier,
    MissingEditOrProviderAuthority,
    MissingCredentials,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingIssueIntakeDecision {
    ReadyToRun,
    NeedsAuthority,
    NeedsClarification,
    SplitRequired,
    BlockedUnsafe,
    MissingCredentials,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingIssueClarifyingQuestion {
    pub reason_code: String,
    pub question: String,
    pub required_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingAuthorityRequirement {
    pub reason_code: String,
    pub summary: String,
    pub required_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingVerifierPlan {
    pub plan_kind: String,
    pub summary: String,
    #[serde(default)]
    pub suggested_verifier_commands: Vec<String>,
    #[serde(default)]
    pub missing_inputs: Vec<String>,
    pub next_action: String,
}

impl Default for AutonomousCodingVerifierPlan {
    fn default() -> Self {
        Self {
            plan_kind: "unknown".to_string(),
            summary: "No verifier plan recorded.".to_string(),
            suggested_verifier_commands: Vec::new(),
            missing_inputs: Vec::new(),
            next_action: "Inspect the issue and provide verifier authority.".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingIssueIntakeOutcome {
    pub schema_version: u32,
    pub intake_id: String,
    pub status: AutonomousCodingIssueIntakeStatus,
    pub reason_code: String,
    #[serde(default)]
    pub classification: AutonomousCodingIssueIntakeClassification,
    #[serde(default)]
    pub classification_summary: String,
    #[serde(default)]
    pub decision: AutonomousCodingIssueIntakeDecision,
    #[serde(default)]
    pub clarifying_questions: Vec<AutonomousCodingIssueClarifyingQuestion>,
    pub issue_url: String,
    pub issue_title: String,
    #[serde(default)]
    pub issue_body: String,
    pub issue_body_summary: String,
    pub repo_path: PathBuf,
    pub base_branch: String,
    pub required_authority: Vec<AutonomousCodingAuthorityRequirement>,
    #[serde(default)]
    pub verifier_plan: AutonomousCodingVerifierPlan,
    #[serde(default)]
    pub missing_inputs: Vec<String>,
    #[serde(default)]
    pub next_action_summary: String,
    pub created_unix_ms: u64,
    pub updated_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingIssueToMergeStatus {
    PrReady,
    AutoMergeRequested,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingIssueToMergeRequest {
    pub intake_id: String,
    pub mission_id: String,
    pub session_key: String,
    pub repo_path: PathBuf,
    pub issue_url: String,
    pub issue_title: String,
    pub issue_body: String,
    pub goal: String,
    pub base_branch: String,
    pub branch_prefix: String,
    pub verifier_commands: Vec<String>,
    pub pr_mode: CodingMissionPrMode,
    pub allowed_roots: Vec<PathBuf>,
    #[serde(default)]
    pub controlled_edits: Vec<CodingMissionControlledEdit>,
    #[serde(default)]
    pub provider_repair: AutonomousCodingProviderRepairPolicy,
    pub commit_message: String,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    pub allow_auto_merge: bool,
    pub merge_method: AutonomousCodingMergeMethod,
    pub delete_branch: bool,
    #[serde(default)]
    pub github_env: BTreeMap<String, String>,
    #[serde(default)]
    pub gh_binary: Option<PathBuf>,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingIssueToMergeOutcome {
    pub status: AutonomousCodingIssueToMergeStatus,
    pub reason_code: String,
    #[serde(default)]
    pub intake: Option<AutonomousCodingIssueIntakeOutcome>,
    #[serde(default)]
    pub submit: Option<AutonomousCodingJobSubmitOutcome>,
    #[serde(default)]
    pub run: Option<AutonomousCodingJobRunOutcome>,
    #[serde(default)]
    pub auto_merge: Option<AutonomousCodingAutoMergeOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobRecord {
    pub schema_version: u32,
    pub job_id: String,
    pub mission_id: String,
    #[serde(default)]
    pub background_job_id: Option<String>,
    pub status: AutonomousCodingJobStatus,
    pub reason_code: String,
    pub state_root: PathBuf,
    pub repo_path: PathBuf,
    #[serde(default)]
    pub issue_url: Option<String>,
    pub goal: String,
    pub verifier_commands: Vec<String>,
    pub pr_mode: CodingMissionPrMode,
    #[serde(default)]
    pub controlled_edits: Vec<CodingMissionControlledEdit>,
    #[serde(default)]
    pub provider_repair: AutonomousCodingProviderRepairPolicy,
    pub commit_message: String,
    pub created_unix_ms: u64,
    pub updated_unix_ms: u64,
    pub recovery_count: u64,
    pub replay_count: u64,
    #[serde(default)]
    pub last_background_reason_code: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub auto_merge_evidence: Option<AutonomousCodingAutoMergeEvidence>,
    #[serde(default)]
    pub provider_repair_evidence: Vec<AutonomousCodingProviderRepairEvidence>,
    #[serde(default)]
    pub provider_repair_attempts: u32,
    #[serde(default)]
    pub last_heartbeat_unix_ms: Option<u64>,
    #[serde(default)]
    pub lease_expires_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobStatusSnapshot {
    pub schema_version: u32,
    pub job_id: String,
    pub mission_id: String,
    #[serde(default)]
    pub background_job_id: Option<String>,
    pub status: AutonomousCodingJobStatus,
    pub phase: CodingMissionPhase,
    pub reason_code: String,
    pub repo_path: PathBuf,
    #[serde(default)]
    pub issue_url: Option<String>,
    pub verifier_summary: String,
    pub changed_files: Vec<String>,
    pub resume_command: String,
    #[serde(default)]
    pub operator_state: String,
    #[serde(default)]
    pub operator_next_command: String,
    #[serde(default)]
    pub replay_safe: bool,
    #[serde(default)]
    pub recoverable: bool,
    #[serde(default)]
    pub needs_authority: bool,
    #[serde(default)]
    pub stale_lease: bool,
    #[serde(default)]
    pub mark_blocked_command: String,
    pub pr_state: String,
    #[serde(default)]
    pub pr_ready_command: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub pr_publication_reason_code: Option<String>,
    #[serde(default)]
    pub pr_publication_command: Option<String>,
    #[serde(default)]
    pub pr_publication_stdout_path: Option<PathBuf>,
    #[serde(default)]
    pub pr_publication_stderr_path: Option<PathBuf>,
    #[serde(default)]
    pub pr_publication_exit_status: Option<i32>,
    pub recovery_count: u64,
    pub replay_count: u64,
    #[serde(default)]
    pub last_background_reason_code: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub auto_merge_status: Option<String>,
    #[serde(default)]
    pub auto_merge_reason_code: Option<String>,
    #[serde(default)]
    pub auto_merge_command: Option<String>,
    #[serde(default)]
    pub auto_merge_pr_url: Option<String>,
    #[serde(default)]
    pub provider_repair_status: Option<String>,
    #[serde(default)]
    pub provider_repair_reason_code: Option<String>,
    #[serde(default)]
    pub provider_repair_attempts: u32,
    #[serde(default)]
    pub provider_repair_max_attempts: u32,
    #[serde(default)]
    pub provider_repair_provider: Option<String>,
    #[serde(default)]
    pub provider_repair_model: Option<String>,
    #[serde(default)]
    pub provider_repair_context_path: Option<PathBuf>,
    #[serde(default)]
    pub event_log_path: PathBuf,
    #[serde(default)]
    pub last_heartbeat_unix_ms: Option<u64>,
    #[serde(default)]
    pub lease_expires_unix_ms: Option<u64>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobEvent {
    pub schema_version: u32,
    pub job_id: String,
    pub event: String,
    pub status: AutonomousCodingJobStatus,
    pub reason_code: String,
    pub detail: String,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobSubmitOutcome {
    pub record: AutonomousCodingJobRecord,
    #[serde(default)]
    pub background_job: Option<BackgroundJobRecord>,
    pub status: AutonomousCodingJobStatusSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobRunOutcome {
    pub record: AutonomousCodingJobRecord,
    pub status: AutonomousCodingJobStatusSnapshot,
    #[serde(default)]
    pub pr_ready_bundle: Option<CodingMissionPrReadyBundle>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobMarkBlockedRequest {
    pub job_id: String,
    pub reason_code: String,
    pub detail: String,
    pub started_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingJobMarkBlockedOutcome {
    pub record: AutonomousCodingJobRecord,
    pub status: AutonomousCodingJobStatusSnapshot,
}

#[derive(Debug, Clone)]
pub struct AutonomousCodingJobRecoveryOutcome {
    pub background_report: BackgroundJobRecoveryReport,
    pub recovered_jobs: Vec<AutonomousCodingJobStatusSnapshot>,
}

#[derive(Debug, Clone)]
pub struct AutonomousCodingJobRuntime {
    config: AutonomousCodingJobRuntimeConfig,
    background_runtime: BackgroundJobRuntime,
}

impl AutonomousCodingJobRuntime {
    pub fn new(config: AutonomousCodingJobRuntimeConfig) -> Result<Self> {
        ensure_autonomous_coding_job_layout(&config.state_dir)?;
        let background_runtime = BackgroundJobRuntime::new(BackgroundJobRuntimeConfig {
            state_dir: config.background_jobs_state_dir.clone(),
            default_timeout_ms: config.default_timeout_ms.max(1),
            max_timeout_ms: config.max_timeout_ms.max(config.default_timeout_ms.max(1)),
            worker_poll_ms: config.worker_poll_ms,
            stuck_recovery_poll_ms: config.stuck_recovery_poll_ms,
        })?;
        Ok(Self {
            config,
            background_runtime,
        })
    }

    pub fn config(&self) -> &AutonomousCodingJobRuntimeConfig {
        &self.config
    }

    pub async fn submit_job(
        &self,
        request: AutonomousCodingJobSubmitRequest,
    ) -> Result<AutonomousCodingJobSubmitOutcome> {
        ensure_autonomous_coding_job_layout(&self.config.state_dir)?;
        let job_id = next_autonomous_coding_job_id();
        let mission = CodingMissionState::create(CodingMissionConfig {
            state_root: self.config.state_dir.clone(),
            mission_id: request.mission_id.clone(),
            session_key: request.session_key.clone(),
            repo_path: request.repo_path.clone(),
            issue_url: request.issue_url.clone(),
            goal: request.goal.clone(),
            base_branch: request.base_branch.clone(),
            branch_prefix: request.branch_prefix.clone(),
            verifier_commands: request.verifier_commands.clone(),
            pr_mode: request.pr_mode,
            allowed_roots: request.allowed_roots.clone(),
            created_unix_ms: request.started_unix_ms,
        })?;
        save_coding_mission_state(&mission)?;

        let mut record = AutonomousCodingJobRecord {
            schema_version: AUTONOMOUS_CODING_JOB_SCHEMA_VERSION,
            job_id,
            mission_id: request.mission_id,
            background_job_id: None,
            status: AutonomousCodingJobStatus::Queued,
            reason_code: AUTONOMOUS_CODING_JOB_REASON_QUEUED.to_string(),
            state_root: self.config.state_dir.clone(),
            repo_path: mission.repo_path.clone(),
            issue_url: request.issue_url,
            goal: request.goal,
            verifier_commands: request.verifier_commands,
            pr_mode: request.pr_mode,
            controlled_edits: request.controlled_edits,
            provider_repair: request.provider_repair,
            commit_message: request.commit_message,
            created_unix_ms: request.started_unix_ms,
            updated_unix_ms: request.started_unix_ms,
            recovery_count: 0,
            replay_count: 0,
            last_background_reason_code: None,
            last_error: None,
            auto_merge_evidence: None,
            provider_repair_evidence: Vec::new(),
            provider_repair_attempts: 0,
            last_heartbeat_unix_ms: Some(request.started_unix_ms),
            lease_expires_unix_ms: Some(
                request
                    .started_unix_ms
                    .saturating_add(AUTONOMOUS_CODING_JOB_LEASE_MS),
            ),
        };
        append_autonomous_coding_job_event(
            &record,
            "job_submitted",
            AUTONOMOUS_CODING_JOB_REASON_QUEUED,
            "durable autonomous coding job submitted",
            request.started_unix_ms,
        )?;

        let mut background_job = None;
        if request.enqueue_background_job {
            let created = self
                .background_runtime
                .create_job(BackgroundJobCreateRequest {
                    command: self.config.runner_command.display().to_string(),
                    args: self.runner_args_for_job(record.job_id.as_str()),
                    env: BTreeMap::new(),
                    cwd: Some(record.repo_path.clone()),
                    timeout_ms: request.timeout_ms,
                    trace: BackgroundJobTraceContext::default(),
                })
                .await?;
            record.background_job_id = Some(created.job_id.clone());
            record.last_background_reason_code = Some(created.reason_code.clone());
            background_job = Some(created);
        }

        persist_autonomous_coding_job_record(&record)?;
        let status = self.refresh_status_snapshot(&record)?;
        Ok(AutonomousCodingJobSubmitOutcome {
            record,
            background_job,
            status,
        })
    }

    pub fn run_or_replay_job(
        &self,
        job_id: &str,
        started_unix_ms: u64,
    ) -> Result<AutonomousCodingJobRunOutcome> {
        self.execute_job(
            job_id,
            None,
            None,
            started_unix_ms,
            ReplayCounterMode::Auto,
            AutonomousCodingPrPublicationOptions::default(),
        )
    }

    pub fn replay_job(
        &self,
        request: AutonomousCodingJobReplayRequest,
    ) -> Result<AutonomousCodingJobRunOutcome> {
        self.execute_job(
            request.job_id.as_str(),
            Some(request.controlled_edits),
            request.commit_message,
            request.started_unix_ms,
            ReplayCounterMode::Increment,
            AutonomousCodingPrPublicationOptions::default(),
        )
    }

    pub async fn run_issue_to_merge(
        &self,
        request: AutonomousCodingIssueToMergeRequest,
    ) -> Result<AutonomousCodingIssueToMergeOutcome> {
        ensure_autonomous_coding_job_layout(&self.config.state_dir)?;
        let provider_repair_configured = request.provider_repair.is_configured();
        let verifier_commands = if request.verifier_commands.is_empty() {
            derive_concrete_verifier_commands(
                request.repo_path.as_path(),
                &request.issue_title,
                &request.issue_body,
            )
        } else {
            request.verifier_commands.clone()
        };
        let has_verifier = !verifier_commands.is_empty();
        let has_edit_or_provider_authority =
            !request.controlled_edits.is_empty() || provider_repair_configured;
        let needs_intake = request.verifier_commands.is_empty() || !has_edit_or_provider_authority;
        let mut intake_for_outcome = None;
        if needs_intake {
            let intake = self.intake_issue_with_context(
                AutonomousCodingIssueIntakeRequest {
                    intake_id: request.intake_id.clone(),
                    issue_url: request.issue_url.clone(),
                    issue_title: request.issue_title.clone(),
                    issue_body: request.issue_body.clone(),
                    repo_path: request.repo_path.clone(),
                    base_branch: request.base_branch.clone(),
                    started_unix_ms: request.started_unix_ms,
                },
                IssueIntakeAuthorityContext {
                    has_verifier,
                    has_edit_or_provider_authority,
                    has_required_credentials: true,
                },
            )?;
            if !has_verifier
                || !has_edit_or_provider_authority
                || intake.decision != AutonomousCodingIssueIntakeDecision::ReadyToRun
            {
                return Ok(AutonomousCodingIssueToMergeOutcome {
                    status: AutonomousCodingIssueToMergeStatus::Blocked,
                    reason_code: intake.reason_code.clone(),
                    intake: Some(intake),
                    submit: None,
                    run: None,
                    auto_merge: None,
                });
            }
            intake_for_outcome = Some(intake);
        }

        let allowed_roots = if request.allowed_roots.is_empty() {
            vec![request.repo_path.clone()]
        } else {
            request.allowed_roots.clone()
        };
        let submit = self
            .submit_job(AutonomousCodingJobSubmitRequest {
                mission_id: request.mission_id,
                session_key: request.session_key,
                repo_path: request.repo_path,
                issue_url: Some(request.issue_url),
                goal: request.goal,
                base_branch: request.base_branch,
                branch_prefix: request.branch_prefix,
                verifier_commands,
                pr_mode: request.pr_mode,
                allowed_roots,
                controlled_edits: request.controlled_edits,
                provider_repair: request.provider_repair,
                commit_message: request.commit_message,
                enqueue_background_job: false,
                timeout_ms: request.timeout_ms,
                started_unix_ms: request.started_unix_ms,
            })
            .await?;
        let job_id = submit.record.job_id.clone();
        let run = self.execute_job(
            job_id.as_str(),
            None,
            None,
            request.started_unix_ms.saturating_add(1_000),
            ReplayCounterMode::Auto,
            AutonomousCodingPrPublicationOptions {
                github_env: request.github_env.clone(),
                gh_binary: request.gh_binary.clone(),
            },
        )?;

        if run.status.status != AutonomousCodingJobStatus::PrReady {
            let status = match run.status.status {
                AutonomousCodingJobStatus::Blocked => AutonomousCodingIssueToMergeStatus::Blocked,
                _ => AutonomousCodingIssueToMergeStatus::Failed,
            };
            return Ok(AutonomousCodingIssueToMergeOutcome {
                status,
                reason_code: run.status.reason_code.clone(),
                intake: intake_for_outcome,
                submit: Some(submit),
                run: Some(run),
                auto_merge: None,
            });
        }

        if request.allow_auto_merge {
            let auto_merge = self.request_auto_merge(AutonomousCodingAutoMergeRequest {
                job_id,
                allow_auto_merge: true,
                merge_method: request.merge_method,
                delete_branch: request.delete_branch,
                github_env: request.github_env,
                gh_binary: request.gh_binary,
                started_unix_ms: request.started_unix_ms.saturating_add(2_000),
            })?;
            let status = match auto_merge.evidence.status {
                AutonomousCodingAutoMergeStatus::Requested => {
                    AutonomousCodingIssueToMergeStatus::AutoMergeRequested
                }
                AutonomousCodingAutoMergeStatus::Blocked => {
                    AutonomousCodingIssueToMergeStatus::Blocked
                }
                AutonomousCodingAutoMergeStatus::Failed => {
                    AutonomousCodingIssueToMergeStatus::Failed
                }
            };
            return Ok(AutonomousCodingIssueToMergeOutcome {
                status,
                reason_code: auto_merge.evidence.reason_code.clone(),
                intake: intake_for_outcome,
                submit: Some(submit),
                run: Some(run),
                auto_merge: Some(auto_merge),
            });
        }

        Ok(AutonomousCodingIssueToMergeOutcome {
            status: AutonomousCodingIssueToMergeStatus::PrReady,
            reason_code: run.status.reason_code.clone(),
            intake: intake_for_outcome,
            submit: Some(submit),
            run: Some(run),
            auto_merge: None,
        })
    }

    pub async fn recover_stuck_background_jobs(
        &self,
    ) -> Result<AutonomousCodingJobRecoveryOutcome> {
        let background_report = self.background_runtime.recover_stuck_jobs().await?;
        let mut recovered_jobs = Vec::new();
        if background_report.recovered > 0 {
            let recovered_ids = background_report
                .recovered_job_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            for mut record in load_autonomous_coding_job_records(&self.config.state_dir)? {
                let Some(background_job_id) = record.background_job_id.as_deref() else {
                    continue;
                };
                if !recovered_ids.contains(&background_job_id) {
                    continue;
                }
                record.status = AutonomousCodingJobStatus::Recovering;
                record.reason_code = AUTONOMOUS_CODING_JOB_REASON_RECOVERED.to_string();
                record.updated_unix_ms = current_unix_timestamp_ms();
                record.recovery_count = record.recovery_count.saturating_add(1);
                record.last_background_reason_code =
                    Some("job_recovered_after_stuck_timeout".to_string());
                record.last_error = None;
                persist_autonomous_coding_job_record(&record)?;
                recovered_jobs.push(self.refresh_status_snapshot(&record)?);
            }
        }
        Ok(AutonomousCodingJobRecoveryOutcome {
            background_report,
            recovered_jobs,
        })
    }

    pub fn status(&self, job_id: &str) -> Result<AutonomousCodingJobStatusSnapshot> {
        let record = load_autonomous_coding_job_record(&self.config.state_dir, job_id)?;
        self.refresh_status_snapshot(&record)
    }

    pub fn request_auto_merge(
        &self,
        request: AutonomousCodingAutoMergeRequest,
    ) -> Result<AutonomousCodingAutoMergeOutcome> {
        let mut record =
            load_autonomous_coding_job_record(&self.config.state_dir, &request.job_id)?;
        let state = load_coding_mission_state(&record.state_root, &record.mission_id)?;
        let status = build_status_snapshot(&record, &state);

        if !request.allow_auto_merge {
            return self.block_auto_merge(
                record,
                "auto_merge_policy_disabled",
                "auto-merge policy is disabled for this request",
                status.pr_url,
                request.started_unix_ms,
            );
        }
        if record.status != AutonomousCodingJobStatus::PrReady
            || state.phase != CodingMissionPhase::PrReady
        {
            return self.block_auto_merge(
                record,
                "auto_merge_job_not_pr_ready",
                "job must be pr_ready before auto-merge can be requested",
                status.pr_url,
                request.started_unix_ms,
            );
        }
        let Some(pr_url) = status.pr_url.clone() else {
            return self.block_auto_merge(
                record,
                "auto_merge_missing_pr_url",
                "PR URL is required before auto-merge can be requested",
                None,
                request.started_unix_ms,
            );
        };
        if !github_auth_present_for_auto_merge(&request.github_env) {
            return self.block_auto_merge(
                record,
                "auto_merge_missing_github_auth",
                "GH_TOKEN or GITHUB_TOKEN is required for auto-merge",
                Some(pr_url),
                request.started_unix_ms,
            );
        }

        let gh_binary = request.gh_binary.unwrap_or_else(|| PathBuf::from("gh"));
        let mut argv = vec![
            "pr".to_string(),
            "merge".to_string(),
            pr_url.clone(),
            "--auto".to_string(),
            request.merge_method.gh_flag().to_string(),
        ];
        if request.delete_branch {
            argv.push("--delete-branch".to_string());
        }
        debug_assert!(!argv.iter().any(|arg| arg == "--admin"));

        let artifact_dir = autonomous_coding_job_artifact_dir(&record.state_root, &record.job_id);
        std::fs::create_dir_all(&artifact_dir)?;
        let stdout_path = artifact_dir.join("auto-merge.stdout.log");
        let stderr_path = artifact_dir.join("auto-merge.stderr.log");
        let output = Command::new(&gh_binary)
            .args(&argv)
            .envs(&request.github_env)
            .current_dir(&record.repo_path)
            .output();

        let evidence = match output {
            Ok(output) => {
                std::fs::write(&stdout_path, &output.stdout)?;
                std::fs::write(&stderr_path, &output.stderr)?;
                if output.status.success() {
                    AutonomousCodingAutoMergeEvidence {
                        status: AutonomousCodingAutoMergeStatus::Requested,
                        reason_code: "auto_merge_requested".to_string(),
                        pr_url: Some(pr_url),
                        command_argv: auto_merge_command_argv(&gh_binary, &argv),
                        stdout_path: Some(stdout_path),
                        stderr_path: Some(stderr_path),
                        exit_status: output.status.code(),
                        error_summary: None,
                        created_unix_ms: request.started_unix_ms,
                    }
                } else {
                    AutonomousCodingAutoMergeEvidence {
                        status: AutonomousCodingAutoMergeStatus::Failed,
                        reason_code: "auto_merge_command_failed".to_string(),
                        pr_url: Some(pr_url),
                        command_argv: auto_merge_command_argv(&gh_binary, &argv),
                        stdout_path: Some(stdout_path),
                        stderr_path: Some(stderr_path),
                        exit_status: output.status.code(),
                        error_summary: Some(format!(
                            "gh pr merge exited with status {}",
                            output.status.code().unwrap_or(-1)
                        )),
                        created_unix_ms: request.started_unix_ms,
                    }
                }
            }
            Err(error) => AutonomousCodingAutoMergeEvidence {
                status: AutonomousCodingAutoMergeStatus::Failed,
                reason_code: "auto_merge_command_spawn_failed".to_string(),
                pr_url: Some(pr_url),
                command_argv: auto_merge_command_argv(&gh_binary, &argv),
                stdout_path: None,
                stderr_path: None,
                exit_status: None,
                error_summary: Some(error.to_string()),
                created_unix_ms: request.started_unix_ms,
            },
        };

        record.reason_code = evidence.reason_code.clone();
        record.updated_unix_ms = request.started_unix_ms;
        record.auto_merge_evidence = Some(evidence.clone());
        persist_autonomous_coding_job_record(&record)?;
        let status = self.refresh_status_snapshot(&record)?;
        Ok(AutonomousCodingAutoMergeOutcome {
            record,
            status,
            evidence,
        })
    }

    pub fn intake_issue_without_authority(
        &self,
        request: AutonomousCodingIssueIntakeRequest,
    ) -> Result<AutonomousCodingIssueIntakeOutcome> {
        self.intake_issue_with_context(
            request,
            IssueIntakeAuthorityContext {
                has_verifier: false,
                has_edit_or_provider_authority: false,
                has_required_credentials: true,
            },
        )
    }

    fn intake_issue_with_context(
        &self,
        request: AutonomousCodingIssueIntakeRequest,
        context: IssueIntakeAuthorityContext,
    ) -> Result<AutonomousCodingIssueIntakeOutcome> {
        ensure_autonomous_coding_job_layout(&self.config.state_dir)?;
        let repo_path = canonicalize_existing_dir(request.repo_path.as_path())?;
        let classification =
            classify_issue_intake(&request.issue_title, &request.issue_body, context);
        let verifier_plan = issue_intake_verifier_plan(
            repo_path.as_path(),
            &request.issue_title,
            &request.issue_body,
            classification.classification,
            context,
        );
        let decision = issue_intake_decision(classification.classification);
        let clarifying_questions = issue_intake_clarifying_questions(
            classification.classification,
            context,
            &verifier_plan,
        );
        let missing_inputs = verifier_plan.missing_inputs.clone();
        let next_action_summary = verifier_plan.next_action.clone();
        let issue_body_summary = summarize_issue_body(&request.issue_body);
        let outcome = AutonomousCodingIssueIntakeOutcome {
            schema_version: AUTONOMOUS_CODING_JOB_SCHEMA_VERSION,
            intake_id: request.intake_id,
            status: AutonomousCodingIssueIntakeStatus::Blocked,
            reason_code: issue_intake_reason_code(classification.classification).to_string(),
            classification: classification.classification,
            classification_summary: classification.summary,
            decision,
            clarifying_questions,
            issue_url: request.issue_url,
            issue_title: request.issue_title,
            issue_body: request.issue_body,
            issue_body_summary,
            repo_path,
            base_branch: request.base_branch,
            required_authority: issue_intake_required_authority(context),
            verifier_plan,
            missing_inputs,
            next_action_summary,
            created_unix_ms: request.started_unix_ms,
            updated_unix_ms: request.started_unix_ms,
        };
        persist_autonomous_coding_issue_intake(&self.config.state_dir, &outcome)?;
        Ok(outcome)
    }

    pub fn issue_intake_status(
        &self,
        intake_id: &str,
    ) -> Result<AutonomousCodingIssueIntakeOutcome> {
        load_autonomous_coding_issue_intake(&self.config.state_dir, intake_id)
    }

    pub fn mark_job_blocked(
        &self,
        request: AutonomousCodingJobMarkBlockedRequest,
    ) -> Result<AutonomousCodingJobMarkBlockedOutcome> {
        let mut record =
            load_autonomous_coding_job_record(&self.config.state_dir, &request.job_id)?;
        let reason_code = request.reason_code.trim();
        let reason_code = if reason_code.is_empty() {
            "operator_marked_blocked"
        } else {
            reason_code
        };
        let detail = request.detail.trim();
        let detail = if detail.is_empty() {
            "operator marked job blocked"
        } else {
            detail
        };
        record.status = AutonomousCodingJobStatus::Blocked;
        record.reason_code = reason_code.to_string();
        record.last_error = Some(detail.to_string());
        record.lease_expires_unix_ms = None;
        record.updated_unix_ms = request.started_unix_ms;
        persist_autonomous_coding_job_record(&record)?;
        append_autonomous_coding_job_event(
            &record,
            "job_marked_blocked",
            reason_code,
            detail,
            request.started_unix_ms,
        )?;
        let status = self.refresh_status_snapshot(&record)?;
        Ok(AutonomousCodingJobMarkBlockedOutcome { record, status })
    }

    fn block_auto_merge(
        &self,
        mut record: AutonomousCodingJobRecord,
        reason_code: &str,
        error_summary: &str,
        pr_url: Option<String>,
        started_unix_ms: u64,
    ) -> Result<AutonomousCodingAutoMergeOutcome> {
        let evidence = AutonomousCodingAutoMergeEvidence {
            status: AutonomousCodingAutoMergeStatus::Blocked,
            reason_code: reason_code.to_string(),
            pr_url,
            command_argv: Vec::new(),
            stdout_path: None,
            stderr_path: None,
            exit_status: None,
            error_summary: Some(error_summary.to_string()),
            created_unix_ms: started_unix_ms,
        };
        record.reason_code = reason_code.to_string();
        record.updated_unix_ms = started_unix_ms;
        record.auto_merge_evidence = Some(evidence.clone());
        persist_autonomous_coding_job_record(&record)?;
        let status = self.refresh_status_snapshot(&record)?;
        Ok(AutonomousCodingAutoMergeOutcome {
            record,
            status,
            evidence,
        })
    }

    fn execute_job(
        &self,
        job_id: &str,
        controlled_edits_override: Option<Vec<CodingMissionControlledEdit>>,
        commit_message_override: Option<String>,
        started_unix_ms: u64,
        replay_counter_mode: ReplayCounterMode,
        pr_publication: AutonomousCodingPrPublicationOptions,
    ) -> Result<AutonomousCodingJobRunOutcome> {
        let mut record = load_autonomous_coding_job_record(&self.config.state_dir, job_id)?;
        let mut controlled_edits =
            controlled_edits_override.unwrap_or_else(|| record.controlled_edits.clone());
        if !controlled_edits.is_empty() {
            record.controlled_edits = controlled_edits.clone();
        }
        if let Some(commit_message) = commit_message_override {
            if !commit_message.trim().is_empty() {
                record.commit_message = commit_message;
            }
        }

        let state_before = load_coding_mission_state(&record.state_root, &record.mission_id)?;
        let should_count_replay = match replay_counter_mode {
            ReplayCounterMode::Increment => true,
            ReplayCounterMode::Auto => {
                state_before.resume_checkpoint.is_some()
                    && state_before.phase != CodingMissionPhase::Intake
            }
        };
        if should_count_replay {
            record.replay_count = record.replay_count.saturating_add(1);
        }
        record.status = AutonomousCodingJobStatus::Running;
        record.reason_code = AUTONOMOUS_CODING_JOB_REASON_RUNNING.to_string();
        record.updated_unix_ms = started_unix_ms;
        record.last_heartbeat_unix_ms = Some(started_unix_ms);
        record.lease_expires_unix_ms =
            Some(started_unix_ms.saturating_add(AUTONOMOUS_CODING_JOB_LEASE_MS));
        record.last_error = None;
        persist_autonomous_coding_job_record(&record)?;
        append_autonomous_coding_job_event(
            &record,
            "job_run_started",
            AUTONOMOUS_CODING_JOB_REASON_RUNNING,
            "durable autonomous coding job run started",
            started_unix_ms,
        )?;
        self.refresh_status_snapshot(&record)?;

        let runner = CodingMissionRunner::new();
        let mut pr_ready_bundle = None;
        let mut last_phase = state_before.phase;
        let mut final_state = state_before;
        for pass in 0..AUTONOMOUS_CODING_JOB_MAX_REPLAY_PASSES {
            let pass_started = started_unix_ms.saturating_add((pass as u64).saturating_mul(1_000));
            record.last_heartbeat_unix_ms = Some(pass_started);
            record.lease_expires_unix_ms =
                Some(pass_started.saturating_add(AUTONOMOUS_CODING_JOB_LEASE_MS));
            persist_autonomous_coding_job_record(&record)?;
            let outcome = if final_state.phase == CodingMissionPhase::Intake {
                runner.run(
                    &mut final_state,
                    CodingMissionRunRequest {
                        controlled_edit: None,
                        controlled_edits: controlled_edits.clone(),
                        commit_message: record.commit_message.clone(),
                        started_unix_ms: pass_started,
                    },
                )?
            } else {
                let resumed = runner.resume(CodingMissionResumeRequest {
                    state_root: record.state_root.clone(),
                    mission_id: record.mission_id.clone(),
                    controlled_edit: None,
                    controlled_edits: controlled_edits.clone(),
                    commit_message: record.commit_message.clone(),
                    started_unix_ms: pass_started,
                    stop_after: None,
                })?;
                resumed.run
            };

            final_state = load_coding_mission_state(&record.state_root, &record.mission_id)?;
            if final_state.phase == CodingMissionPhase::PrReady {
                pr_ready_bundle = Some(ensure_pr_ready_bundle(
                    &mut final_state,
                    &record,
                    pass_started.saturating_add(900),
                    &pr_publication,
                )?);
                record.status = AutonomousCodingJobStatus::PrReady;
                record.reason_code = AUTONOMOUS_CODING_JOB_REASON_PR_READY.to_string();
                record.updated_unix_ms = pass_started.saturating_add(999);
                record.last_heartbeat_unix_ms = Some(pass_started.saturating_add(999));
                record.lease_expires_unix_ms = None;
                record.last_error = None;
                persist_autonomous_coding_job_record(&record)?;
                append_autonomous_coding_job_event(
                    &record,
                    "job_pr_ready",
                    AUTONOMOUS_CODING_JOB_REASON_PR_READY,
                    "verifier passed and PR-ready evidence exists",
                    pass_started.saturating_add(999),
                )?;
                let status = self.refresh_status_snapshot(&record)?;
                return Ok(AutonomousCodingJobRunOutcome {
                    record,
                    status,
                    pr_ready_bundle,
                });
            }

            if let Some(reason) = outcome.blocked_reason {
                record.status = AutonomousCodingJobStatus::Blocked;
                record.reason_code = AUTONOMOUS_CODING_JOB_REASON_BLOCKED.to_string();
                record.updated_unix_ms = pass_started.saturating_add(999);
                record.last_heartbeat_unix_ms = Some(pass_started.saturating_add(999));
                record.lease_expires_unix_ms = None;
                record.last_error = Some(reason);
                persist_autonomous_coding_job_record(&record)?;
                append_autonomous_coding_job_event(
                    &record,
                    "job_blocked",
                    AUTONOMOUS_CODING_JOB_REASON_BLOCKED,
                    record
                        .last_error
                        .as_deref()
                        .unwrap_or("coding mission blocked"),
                    pass_started.saturating_add(999),
                )?;
                let status = self.refresh_status_snapshot(&record)?;
                return Ok(AutonomousCodingJobRunOutcome {
                    record,
                    status,
                    pr_ready_bundle,
                });
            }

            let apply_edit_checkpoint =
                final_state
                    .resume_checkpoint
                    .as_ref()
                    .is_some_and(|checkpoint| {
                        checkpoint.next_action
                            == tau_agent_core::CodingMissionResumeAction::ApplyEdit
                    });
            if apply_edit_checkpoint && !controlled_edits.is_empty() {
                append_autonomous_coding_job_event(
                    &record,
                    "controlled_edit_verifier_failed",
                    AUTONOMOUS_CODING_JOB_REASON_WAITING_FOR_EDIT,
                    "controlled/provider repair edit did not satisfy verifier; awaiting next edit",
                    pass_started.saturating_add(998),
                )?;
                controlled_edits.clear();
                record.controlled_edits.clear();
                record.updated_unix_ms = pass_started.saturating_add(998);
                record.last_heartbeat_unix_ms = Some(pass_started.saturating_add(998));
                persist_autonomous_coding_job_record(&record)?;
                continue;
            }
            let waiting_for_edit = apply_edit_checkpoint && controlled_edits.is_empty();
            if waiting_for_edit {
                if record.provider_repair.is_configured() {
                    if record.provider_repair_attempts >= record.provider_repair.max_attempts {
                        record.status = AutonomousCodingJobStatus::Blocked;
                        record.reason_code =
                            AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR_EXHAUSTED.to_string();
                        record.updated_unix_ms = pass_started.saturating_add(999);
                        record.last_heartbeat_unix_ms = Some(pass_started.saturating_add(999));
                        record.lease_expires_unix_ms = None;
                        record.last_error = Some("provider repair attempts exhausted".to_string());
                        persist_autonomous_coding_job_record(&record)?;
                        append_autonomous_coding_job_event(
                            &record,
                            "provider_repair_exhausted",
                            AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR_EXHAUSTED,
                            "provider repair attempts exhausted before verifier passed",
                            pass_started.saturating_add(999),
                        )?;
                        let status = self.refresh_status_snapshot(&record)?;
                        return Ok(AutonomousCodingJobRunOutcome {
                            record,
                            status,
                            pr_ready_bundle,
                        });
                    }

                    let attempt_index = record.provider_repair_attempts.saturating_add(1);
                    append_autonomous_coding_job_event(
                        &record,
                        "provider_repair_requested",
                        AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR,
                        "provider repair adapter invoked after verifier failure",
                        pass_started.saturating_add(500),
                    )?;
                    let repair = run_autonomous_coding_provider_repair(
                        &record.provider_repair,
                        &final_state,
                        &record.job_id,
                        attempt_index,
                        &autonomous_coding_job_artifact_dir(&record.state_root, &record.job_id),
                        pass_started.saturating_add(600),
                    )?;
                    record.provider_repair_attempts = attempt_index;
                    record
                        .provider_repair_evidence
                        .push(repair.evidence.clone());
                    record.updated_unix_ms = pass_started.saturating_add(700);
                    record.last_heartbeat_unix_ms = Some(pass_started.saturating_add(700));
                    record.reason_code = repair.evidence.reason_code.clone();
                    if repair.edits.is_empty() {
                        record.last_error = repair.evidence.error_summary.clone().or_else(|| {
                            Some("provider repair produced no applicable edits".to_string())
                        });
                        persist_autonomous_coding_job_record(&record)?;
                        append_autonomous_coding_job_event(
                            &record,
                            "provider_repair_rejected",
                            repair.evidence.reason_code.as_str(),
                            record
                                .last_error
                                .as_deref()
                                .unwrap_or("provider repair produced no applicable edits"),
                            pass_started.saturating_add(700),
                        )?;
                        continue;
                    }
                    controlled_edits = repair.edits;
                    record.controlled_edits = controlled_edits.clone();
                    record.last_error = None;
                    persist_autonomous_coding_job_record(&record)?;
                    append_autonomous_coding_job_event(
                        &record,
                        "provider_repair_ready",
                        repair.evidence.reason_code.as_str(),
                        "provider repair edit set accepted for verifier rerun",
                        pass_started.saturating_add(700),
                    )?;
                    continue;
                }
                record.status = AutonomousCodingJobStatus::Running;
                record.reason_code = AUTONOMOUS_CODING_JOB_REASON_WAITING_FOR_EDIT.to_string();
                record.updated_unix_ms = pass_started.saturating_add(999);
                record.last_heartbeat_unix_ms = Some(pass_started.saturating_add(999));
                persist_autonomous_coding_job_record(&record)?;
                let status = self.refresh_status_snapshot(&record)?;
                return Ok(AutonomousCodingJobRunOutcome {
                    record,
                    status,
                    pr_ready_bundle,
                });
            }

            if final_state.phase == last_phase
                && final_state
                    .resume_checkpoint
                    .as_ref()
                    .is_none_or(|checkpoint| {
                        checkpoint.next_action == tau_agent_core::CodingMissionResumeAction::Blocked
                    })
            {
                break;
            }
            last_phase = final_state.phase;
        }

        record.status = AutonomousCodingJobStatus::Failed;
        record.reason_code = AUTONOMOUS_CODING_JOB_REASON_FAILED.to_string();
        record.updated_unix_ms = started_unix_ms.saturating_add(9_999);
        record.last_heartbeat_unix_ms = Some(started_unix_ms.saturating_add(9_999));
        record.lease_expires_unix_ms = None;
        record.last_error = Some("replay_pass_limit_exhausted".to_string());
        persist_autonomous_coding_job_record(&record)?;
        append_autonomous_coding_job_event(
            &record,
            "job_failed",
            AUTONOMOUS_CODING_JOB_REASON_FAILED,
            "replay pass limit exhausted",
            started_unix_ms.saturating_add(9_999),
        )?;
        let status = self.refresh_status_snapshot(&record)?;
        Ok(AutonomousCodingJobRunOutcome {
            record,
            status,
            pr_ready_bundle,
        })
    }

    fn refresh_status_snapshot(
        &self,
        record: &AutonomousCodingJobRecord,
    ) -> Result<AutonomousCodingJobStatusSnapshot> {
        let state = load_coding_mission_state(&record.state_root, &record.mission_id)?;
        let status = build_status_snapshot(record, &state);
        persist_autonomous_coding_job_status(&self.config.state_dir, &status)?;
        Ok(status)
    }

    fn runner_args_for_job(&self, job_id: &str) -> Vec<String> {
        let mut args = self.config.runner_args_prefix.clone();
        args.extend([
            "run".to_string(),
            "--state-dir".to_string(),
            self.config.state_dir.display().to_string(),
            "--job-id".to_string(),
            job_id.to_string(),
        ]);
        args
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplayCounterMode {
    Auto,
    Increment,
}

#[derive(Debug, Clone, Default)]
struct AutonomousCodingPrPublicationOptions {
    github_env: BTreeMap<String, String>,
    gh_binary: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
struct IssueIntakeAuthorityContext {
    has_verifier: bool,
    has_edit_or_provider_authority: bool,
    has_required_credentials: bool,
}

#[derive(Debug, Clone)]
struct IssueIntakeClassificationResult {
    classification: AutonomousCodingIssueIntakeClassification,
    summary: String,
}

#[derive(Debug, Clone)]
struct AutonomousCodingOperatorStatus {
    state: String,
    next_command: String,
    replay_safe: bool,
    recoverable: bool,
    needs_authority: bool,
    stale_lease: bool,
    mark_blocked_command: String,
}

pub fn autonomous_coding_job_record_path(state_dir: &Path, job_id: &str) -> PathBuf {
    state_dir
        .join("autonomous-coding-jobs")
        .join(format!("{job_id}.json"))
}

pub fn autonomous_coding_job_status_path(state_dir: &Path, job_id: &str) -> PathBuf {
    state_dir
        .join("autonomous-coding-jobs")
        .join(format!("{job_id}.status.json"))
}

pub fn autonomous_coding_job_events_path(state_dir: &Path, job_id: &str) -> PathBuf {
    state_dir
        .join("autonomous-coding-jobs")
        .join(job_id)
        .join("events.jsonl")
}

pub fn autonomous_coding_issue_intake_path(state_dir: &Path, intake_id: &str) -> PathBuf {
    state_dir
        .join("issue-intake")
        .join(format!("{intake_id}.json"))
}

fn autonomous_coding_job_artifact_dir(state_dir: &Path, job_id: &str) -> PathBuf {
    state_dir.join("autonomous-coding-jobs").join(job_id)
}

fn ensure_autonomous_coding_job_layout(state_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(state_dir.join("autonomous-coding-jobs"))?;
    std::fs::create_dir_all(state_dir.join("coding-missions"))?;
    std::fs::create_dir_all(state_dir.join("issue-intake"))?;
    Ok(())
}

fn next_autonomous_coding_job_id() -> String {
    let sequence = NEXT_AUTONOMOUS_CODING_JOB_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    format!(
        "coding-job-{}-{sequence}",
        current_unix_timestamp_ms().max(1)
    )
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn persist_autonomous_coding_job_record(record: &AutonomousCodingJobRecord) -> Result<()> {
    let path = autonomous_coding_job_record_path(&record.state_root, &record.job_id);
    write_json_atomic(&path, record)
}

fn persist_autonomous_coding_job_status(
    state_dir: &Path,
    status: &AutonomousCodingJobStatusSnapshot,
) -> Result<()> {
    let path = autonomous_coding_job_status_path(state_dir, &status.job_id);
    write_json_atomic(&path, status)
}

fn append_autonomous_coding_job_event(
    record: &AutonomousCodingJobRecord,
    event: &str,
    reason_code: &str,
    detail: &str,
    created_unix_ms: u64,
) -> Result<()> {
    let event_record = AutonomousCodingJobEvent {
        schema_version: AUTONOMOUS_CODING_JOB_SCHEMA_VERSION,
        job_id: record.job_id.clone(),
        event: event.to_string(),
        status: record.status,
        reason_code: reason_code.to_string(),
        detail: detail.to_string(),
        created_unix_ms,
    };
    let path = autonomous_coding_job_events_path(&record.state_root, &record.job_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut line = serde_json::to_string(&event_record)?;
    line.push('\n');
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?
        .write_all(line.as_bytes())
        .with_context(|| format!("failed to append {}", path.display()))
}

fn persist_autonomous_coding_issue_intake(
    state_dir: &Path,
    outcome: &AutonomousCodingIssueIntakeOutcome,
) -> Result<()> {
    let path = autonomous_coding_issue_intake_path(state_dir, &outcome.intake_id);
    write_json_atomic(&path, outcome)
}

fn load_autonomous_coding_issue_intake(
    state_dir: &Path,
    intake_id: &str,
) -> Result<AutonomousCodingIssueIntakeOutcome> {
    let path = autonomous_coding_issue_intake_path(state_dir, intake_id);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let outcome = serde_json::from_str::<AutonomousCodingIssueIntakeOutcome>(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    if outcome.schema_version != AUTONOMOUS_CODING_JOB_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported issue intake schema {} at {}; expected {}",
            outcome.schema_version,
            path.display(),
            AUTONOMOUS_CODING_JOB_SCHEMA_VERSION
        ));
    }
    Ok(outcome)
}

fn load_autonomous_coding_job_record(
    state_dir: &Path,
    job_id: &str,
) -> Result<AutonomousCodingJobRecord> {
    let path = autonomous_coding_job_record_path(state_dir, job_id);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let record = serde_json::from_str::<AutonomousCodingJobRecord>(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    if record.schema_version != AUTONOMOUS_CODING_JOB_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported autonomous coding job schema {} at {}; expected {}",
            record.schema_version,
            path.display(),
            AUTONOMOUS_CODING_JOB_SCHEMA_VERSION
        ));
    }
    Ok(record)
}

fn load_autonomous_coding_job_records(state_dir: &Path) -> Result<Vec<AutonomousCodingJobRecord>> {
    let jobs_dir = state_dir.join("autonomous-coding-jobs");
    if !jobs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&jobs_dir)
        .with_context(|| format!("failed to read {}", jobs_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(".status.json"))
        {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        records.push(
            serde_json::from_str::<AutonomousCodingJobRecord>(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?,
        );
    }
    Ok(records)
}

fn build_operator_status(
    record: &AutonomousCodingJobRecord,
    state: &CodingMissionState,
) -> AutonomousCodingOperatorStatus {
    let stale_lease = record
        .lease_expires_unix_ms
        .is_some_and(|expires| expires <= current_unix_timestamp_ms())
        && matches!(
            record.status,
            AutonomousCodingJobStatus::Queued
                | AutonomousCodingJobStatus::Running
                | AutonomousCodingJobStatus::Recovering
        );
    let replay_safe = state.resume_checkpoint.is_some()
        && matches!(
            record.status,
            AutonomousCodingJobStatus::Running
                | AutonomousCodingJobStatus::Recovering
                | AutonomousCodingJobStatus::Blocked
                | AutonomousCodingJobStatus::Failed
        );
    let needs_authority = record.status == AutonomousCodingJobStatus::Blocked
        && matches!(
            record.reason_code.as_str(),
            AUTONOMOUS_CODING_JOB_REASON_WAITING_FOR_EDIT
                | AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR_EXHAUSTED
        );
    let recoverable = stale_lease || record.status == AutonomousCodingJobStatus::Recovering;
    let mark_blocked_command = autonomous_coding_mark_blocked_command(record);

    if stale_lease {
        return AutonomousCodingOperatorStatus {
            state: "stale_lease".to_string(),
            next_command: autonomous_coding_recover_command(record),
            replay_safe,
            recoverable: true,
            needs_authority,
            stale_lease,
            mark_blocked_command,
        };
    }

    if needs_authority {
        return AutonomousCodingOperatorStatus {
            state: "needs_authority".to_string(),
            next_command: if replay_safe {
                autonomous_coding_replay_command(record)
            } else {
                mark_blocked_command.clone()
            },
            replay_safe,
            recoverable,
            needs_authority: true,
            stale_lease,
            mark_blocked_command,
        };
    }

    if replay_safe && record.status == AutonomousCodingJobStatus::Failed {
        return AutonomousCodingOperatorStatus {
            state: "safe_to_replay".to_string(),
            next_command: autonomous_coding_replay_command(record),
            replay_safe,
            recoverable,
            needs_authority,
            stale_lease,
            mark_blocked_command,
        };
    }

    let state_label = match record.status {
        AutonomousCodingJobStatus::Queued
        | AutonomousCodingJobStatus::Running
        | AutonomousCodingJobStatus::Recovering => "running",
        AutonomousCodingJobStatus::PrReady => "complete",
        AutonomousCodingJobStatus::Blocked => "blocked",
        AutonomousCodingJobStatus::Failed => "failed",
    };
    let next_command = match record.status {
        AutonomousCodingJobStatus::Queued
        | AutonomousCodingJobStatus::Running
        | AutonomousCodingJobStatus::Recovering => autonomous_coding_status_command(record),
        AutonomousCodingJobStatus::Blocked | AutonomousCodingJobStatus::Failed => {
            mark_blocked_command.clone()
        }
        AutonomousCodingJobStatus::PrReady => "none".to_string(),
    };

    AutonomousCodingOperatorStatus {
        state: state_label.to_string(),
        next_command,
        replay_safe,
        recoverable,
        needs_authority,
        stale_lease,
        mark_blocked_command,
    }
}

fn autonomous_coding_status_command(record: &AutonomousCodingJobRecord) -> String {
    format!(
        "tau-autonomous-coding-job status --state-dir {} --job-id {}",
        record.state_root.display(),
        record.job_id
    )
}

fn autonomous_coding_replay_command(record: &AutonomousCodingJobRecord) -> String {
    format!(
        "tau-autonomous-coding-job replay --state-dir {} --job-id {}",
        record.state_root.display(),
        record.job_id
    )
}

fn autonomous_coding_recover_command(record: &AutonomousCodingJobRecord) -> String {
    format!(
        "tau-autonomous-coding-job recover --state-dir {}",
        record.state_root.display()
    )
}

fn autonomous_coding_mark_blocked_command(record: &AutonomousCodingJobRecord) -> String {
    format!(
        "tau-autonomous-coding-job mark-blocked --state-dir {} --job-id {} --reason-code operator_marked_blocked",
        record.state_root.display(),
        record.job_id
    )
}

fn build_status_snapshot(
    record: &AutonomousCodingJobRecord,
    state: &CodingMissionState,
) -> AutonomousCodingJobStatusSnapshot {
    let pr_ready_bundle = state.pr_ready_bundle.as_ref();
    let changed_files = pr_ready_bundle
        .map(|bundle| bundle.changed_files.clone())
        .or_else(|| {
            state
                .git_evidence
                .iter()
                .rev()
                .find(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated)
                .map(|evidence| evidence.changed_files.clone())
        })
        .unwrap_or_default();
    let verifier_summary = state
        .command_evidence
        .iter()
        .rev()
        .find(|evidence| evidence.reason_code.starts_with("coding_verifier"))
        .map(|evidence| {
            format!(
                "{}:{}",
                workspace_status_label(evidence.status),
                evidence.reason_code
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let resume_command = state
        .resume_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.operator_resume_command.clone())
        .unwrap_or_else(|| "none".to_string());
    let pr_state = pr_ready_bundle
        .map(|bundle| pr_publication_status_label(bundle.status).to_string())
        .unwrap_or_else(|| {
            if state.phase == CodingMissionPhase::PrReady {
                "pr_ready".to_string()
            } else {
                "none".to_string()
            }
        });
    let latest_repair = record.provider_repair_evidence.last();
    let event_log_path = autonomous_coding_job_events_path(&record.state_root, &record.job_id);
    let operator_status = build_operator_status(record, state);
    AutonomousCodingJobStatusSnapshot {
        schema_version: AUTONOMOUS_CODING_JOB_SCHEMA_VERSION,
        job_id: record.job_id.clone(),
        mission_id: record.mission_id.clone(),
        background_job_id: record.background_job_id.clone(),
        status: record.status,
        phase: state.phase,
        reason_code: record.reason_code.clone(),
        repo_path: record.repo_path.clone(),
        issue_url: record.issue_url.clone(),
        verifier_summary,
        changed_files,
        resume_command,
        operator_state: operator_status.state,
        operator_next_command: operator_status.next_command,
        replay_safe: operator_status.replay_safe,
        recoverable: operator_status.recoverable,
        needs_authority: operator_status.needs_authority,
        stale_lease: operator_status.stale_lease,
        mark_blocked_command: operator_status.mark_blocked_command,
        pr_state,
        pr_ready_command: pr_ready_bundle.map(|bundle| bundle.manual_gh_pr_create_command.clone()),
        pr_url: pr_ready_bundle.and_then(|bundle| bundle.pr_url.clone()),
        pr_publication_reason_code: pr_ready_bundle
            .and_then(|bundle| bundle.draft_pr_reason_code.clone()),
        pr_publication_command: pr_ready_bundle
            .filter(|bundle| !bundle.draft_pr_command_argv.is_empty())
            .map(|bundle| bundle.draft_pr_command_argv.join(" ")),
        pr_publication_stdout_path: pr_ready_bundle
            .and_then(|bundle| bundle.draft_pr_stdout_path.clone()),
        pr_publication_stderr_path: pr_ready_bundle
            .and_then(|bundle| bundle.draft_pr_stderr_path.clone()),
        pr_publication_exit_status: pr_ready_bundle.and_then(|bundle| bundle.draft_pr_exit_status),
        recovery_count: record.recovery_count,
        replay_count: record.replay_count,
        last_background_reason_code: record.last_background_reason_code.clone(),
        last_error: record.last_error.clone(),
        auto_merge_status: record
            .auto_merge_evidence
            .as_ref()
            .map(|evidence| auto_merge_status_label(evidence.status).to_string()),
        auto_merge_reason_code: record
            .auto_merge_evidence
            .as_ref()
            .map(|evidence| evidence.reason_code.clone()),
        auto_merge_command: record
            .auto_merge_evidence
            .as_ref()
            .filter(|evidence| !evidence.command_argv.is_empty())
            .map(|evidence| evidence.command_argv.join(" ")),
        auto_merge_pr_url: record
            .auto_merge_evidence
            .as_ref()
            .and_then(|evidence| evidence.pr_url.clone()),
        provider_repair_status: latest_repair
            .map(|evidence| provider_repair_status_label(evidence.status).to_string()),
        provider_repair_reason_code: latest_repair.map(|evidence| evidence.reason_code.clone()),
        provider_repair_attempts: record.provider_repair_attempts,
        provider_repair_max_attempts: record.provider_repair.max_attempts,
        provider_repair_provider: record.provider_repair.provider.clone(),
        provider_repair_model: record.provider_repair.model.clone(),
        provider_repair_context_path: latest_repair.map(|evidence| evidence.context_path.clone()),
        event_log_path: event_log_path.clone(),
        last_heartbeat_unix_ms: record.last_heartbeat_unix_ms,
        lease_expires_unix_ms: record.lease_expires_unix_ms,
        metadata: BTreeMap::from([
            (
                "mission_state_path".to_string(),
                coding_mission_state_path(&record.state_root, &record.mission_id)
                    .display()
                    .to_string(),
            ),
            (
                "job_record_path".to_string(),
                autonomous_coding_job_record_path(&record.state_root, &record.job_id)
                    .display()
                    .to_string(),
            ),
            (
                "job_event_log_path".to_string(),
                event_log_path.display().to_string(),
            ),
        ]),
    }
}

fn github_auth_present_for_auto_merge(env: &BTreeMap<String, String>) -> bool {
    env.get("GH_TOKEN")
        .or_else(|| env.get("GITHUB_TOKEN"))
        .is_some_and(|value| !value.trim().is_empty())
        || std::env::var("GH_TOKEN").is_ok_and(|value| !value.trim().is_empty())
        || std::env::var("GITHUB_TOKEN").is_ok_and(|value| !value.trim().is_empty())
}

fn auto_merge_command_argv(gh_binary: &Path, args: &[String]) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len().saturating_add(1));
    argv.push(gh_binary.display().to_string());
    argv.extend(args.iter().cloned());
    argv
}

fn auto_merge_status_label(status: AutonomousCodingAutoMergeStatus) -> &'static str {
    match status {
        AutonomousCodingAutoMergeStatus::Requested => "requested",
        AutonomousCodingAutoMergeStatus::Blocked => "blocked",
        AutonomousCodingAutoMergeStatus::Failed => "failed",
    }
}

fn canonicalize_existing_dir(path: &Path) -> Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)
        .with_context(|| format!("failed to canonicalize {}", path.display()))?;
    if !canonical.is_dir() {
        return Err(anyhow!("{} must be a directory", canonical.display()));
    }
    Ok(canonical)
}

fn summarize_issue_body(body: &str) -> String {
    let summary = body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("no issue body supplied");
    if summary.chars().count() <= 240 {
        summary.to_string()
    } else {
        summary.chars().take(240).collect()
    }
}

fn classify_issue_intake(
    title: &str,
    body: &str,
    context: IssueIntakeAuthorityContext,
) -> IssueIntakeClassificationResult {
    let combined = format!("{title}\n{body}");
    let normalized = combined.to_ascii_lowercase();
    let word_count = combined.split_whitespace().count();

    if contains_any(
        &normalized,
        &[
            "bypass branch protection",
            "--admin",
            "force push",
            "force-push",
            "commit secret",
            "leak secret",
            "exfiltrate",
        ],
    ) {
        return IssueIntakeClassificationResult {
            classification: AutonomousCodingIssueIntakeClassification::Unsafe,
            summary: "Issue requests unsafe authority or protected-branch bypass.".to_string(),
        };
    }

    if contains_any(
        &normalized,
        &[
            "all issues",
            "any issue",
            "everything",
            "entire repo",
            "whole repo",
            "fully autonomous forever",
        ],
    ) {
        return IssueIntakeClassificationResult {
            classification: AutonomousCodingIssueIntakeClassification::TooBroad,
            summary: "Issue scope is too broad for a bounded verifier-gated coding job."
                .to_string(),
        };
    }

    if word_count < 6 || body.trim().len() < 16 {
        return IssueIntakeClassificationResult {
            classification: AutonomousCodingIssueIntakeClassification::Underspecified,
            summary: "Issue lacks enough concrete behavior or acceptance detail.".to_string(),
        };
    }

    if !context.has_verifier {
        return IssueIntakeClassificationResult {
            classification: AutonomousCodingIssueIntakeClassification::MissingVerifier,
            summary: "A verifier command or acceptance test is required before mutation."
                .to_string(),
        };
    }

    if !context.has_edit_or_provider_authority {
        return IssueIntakeClassificationResult {
            classification: AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority,
            summary: "Edit authority, provider repair authority, or controlled edits are required before mutation."
                .to_string(),
        };
    }

    if !context.has_required_credentials {
        return IssueIntakeClassificationResult {
            classification: AutonomousCodingIssueIntakeClassification::MissingCredentials,
            summary: "Required provider or GitHub credentials are missing.".to_string(),
        };
    }

    IssueIntakeClassificationResult {
        classification: AutonomousCodingIssueIntakeClassification::Solvable,
        summary: "Issue has enough authority to enter the verifier-gated coding loop.".to_string(),
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn issue_intake_reason_code(
    classification: AutonomousCodingIssueIntakeClassification,
) -> &'static str {
    match classification {
        AutonomousCodingIssueIntakeClassification::Ready
        | AutonomousCodingIssueIntakeClassification::Solvable
        | AutonomousCodingIssueIntakeClassification::MissingVerifier
        | AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority => {
            "issue_intake_authority_required"
        }
        AutonomousCodingIssueIntakeClassification::Underspecified => "issue_intake_underspecified",
        AutonomousCodingIssueIntakeClassification::Unsafe => "issue_intake_unsafe",
        AutonomousCodingIssueIntakeClassification::TooBroad => "issue_intake_too_broad",
        AutonomousCodingIssueIntakeClassification::MissingCredentials => {
            "issue_intake_missing_credentials"
        }
    }
}

fn issue_intake_decision(
    classification: AutonomousCodingIssueIntakeClassification,
) -> AutonomousCodingIssueIntakeDecision {
    match classification {
        AutonomousCodingIssueIntakeClassification::Ready
        | AutonomousCodingIssueIntakeClassification::Solvable => {
            AutonomousCodingIssueIntakeDecision::ReadyToRun
        }
        AutonomousCodingIssueIntakeClassification::MissingVerifier
        | AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority => {
            AutonomousCodingIssueIntakeDecision::NeedsAuthority
        }
        AutonomousCodingIssueIntakeClassification::Underspecified => {
            AutonomousCodingIssueIntakeDecision::NeedsClarification
        }
        AutonomousCodingIssueIntakeClassification::TooBroad => {
            AutonomousCodingIssueIntakeDecision::SplitRequired
        }
        AutonomousCodingIssueIntakeClassification::Unsafe => {
            AutonomousCodingIssueIntakeDecision::BlockedUnsafe
        }
        AutonomousCodingIssueIntakeClassification::MissingCredentials => {
            AutonomousCodingIssueIntakeDecision::MissingCredentials
        }
    }
}

fn issue_intake_clarifying_questions(
    classification: AutonomousCodingIssueIntakeClassification,
    context: IssueIntakeAuthorityContext,
    verifier_plan: &AutonomousCodingVerifierPlan,
) -> Vec<AutonomousCodingIssueClarifyingQuestion> {
    match classification {
        AutonomousCodingIssueIntakeClassification::Unsafe => vec![intake_question(
            "safe_workflow",
            "Can this be restated as a normal branch and PR change without protected-branch bypass, force-push, secret handling, or exfiltration?",
            "safe branch/PR workflow",
        )],
        AutonomousCodingIssueIntakeClassification::TooBroad => vec![
            intake_question(
                "bounded_surface",
                "Which one module, crate, route, command, or file set should this autonomous job change?",
                "single affected surface",
            ),
            intake_question(
                "single_acceptance_criterion",
                "What one acceptance criterion should prove this job is complete?",
                "one verifier-gated acceptance criterion",
            ),
            intake_question(
                "focused_verifier_command",
                "What focused verifier command should Tau run before broader validation?",
                "focused verifier command",
            ),
        ],
        AutonomousCodingIssueIntakeClassification::Underspecified => vec![
            intake_question(
                "expected_behavior",
                "What should happen when the issue is fixed?",
                "expected behavior",
            ),
            intake_question(
                "current_behavior",
                "What currently happens, including any reproduction command, error text, or failing test?",
                "current failing behavior or reproduction",
            ),
            intake_question(
                "affected_surface",
                "Which file, crate, command, route, or product surface is affected?",
                "affected surface",
            ),
            intake_question(
                "verifier_command",
                "Which exact command should Tau run to prove the fix?",
                "acceptance test or verifier command",
            ),
        ],
        AutonomousCodingIssueIntakeClassification::MissingVerifier
        | AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority
        | AutonomousCodingIssueIntakeClassification::MissingCredentials => {
            let mut questions = Vec::new();
            if !context.has_verifier {
                questions.push(intake_question(
                    "verifier_command",
                    "Which exact command should Tau run to prove the fix before mutation?",
                    "--verifier-command or spec-derived test command",
                ));
                append_repo_aware_verifier_questions(&mut questions, verifier_plan);
            }
            if !context.has_edit_or_provider_authority {
                questions.push(intake_question(
                    "mutation_authority",
                    "Should Tau use provider repair, controlled edits, or wait for an operator-supplied patch?",
                    "--edit, --provider-repair-openrouter, --provider-repair-command, or approved mutation authority",
                ));
            }
            if !context.has_required_credentials {
                questions.push(intake_question(
                    "credential_source",
                    "Which credential source should Tau use for provider repair or draft PR creation?",
                    "OPENROUTER_API_KEY, GH_TOKEN, GITHUB_TOKEN, or provider-specific credential",
                ));
            }
            questions
        }
        AutonomousCodingIssueIntakeClassification::Ready
        | AutonomousCodingIssueIntakeClassification::Solvable => Vec::new(),
    }
}

fn append_repo_aware_verifier_questions(
    questions: &mut Vec<AutonomousCodingIssueClarifyingQuestion>,
    verifier_plan: &AutonomousCodingVerifierPlan,
) {
    for missing_input in &verifier_plan.missing_inputs {
        if missing_input == REPO_AWARE_TEST_FILTER_REQUIRED_INPUT {
            push_unique_intake_question(
                questions,
                intake_question(
                    "repo_aware_test_filter",
                    "Which exact quoted or backticked safe test filter should Tau use for the focused Cargo verifier?",
                    missing_input,
                ),
            );
        } else if missing_input == REPO_AWARE_CARGO_METADATA_REQUIRED_INPUT
            || missing_input.starts_with(REPO_AWARE_CARGO_PACKAGE_REQUIRED_PREFIX)
        {
            push_unique_intake_question(
                questions,
                intake_question(
                    "repo_aware_cargo_package",
                    "Which actual Cargo package from this repository should Tau verify?",
                    missing_input,
                ),
            );
        }
    }
}

fn push_unique_intake_question(
    questions: &mut Vec<AutonomousCodingIssueClarifyingQuestion>,
    question: AutonomousCodingIssueClarifyingQuestion,
) {
    if !questions
        .iter()
        .any(|existing| existing.reason_code == question.reason_code)
    {
        questions.push(question);
    }
}

fn intake_question(
    reason_code: &str,
    question: &str,
    required_input: &str,
) -> AutonomousCodingIssueClarifyingQuestion {
    AutonomousCodingIssueClarifyingQuestion {
        reason_code: reason_code.to_string(),
        question: question.to_string(),
        required_input: required_input.to_string(),
    }
}

fn issue_intake_required_authority(
    context: IssueIntakeAuthorityContext,
) -> Vec<AutonomousCodingAuthorityRequirement> {
    let mut required = Vec::new();
    if !context.has_verifier {
        required.push(AutonomousCodingAuthorityRequirement {
            reason_code: "verifier_authority_required".to_string(),
            summary:
                "A verifier command or acceptance test must be provided before code can be changed."
                    .to_string(),
            required_input: "--verifier-command or spec-derived test command".to_string(),
        });
    }
    if !context.has_edit_or_provider_authority {
        required.push(AutonomousCodingAuthorityRequirement {
            reason_code: "edit_authority_required".to_string(),
            summary: "An edit plan, provider edit authority, or controlled edit set must be provided before mutation.".to_string(),
            required_input: "--edit, --provider-repair-openrouter, --provider-repair-command, or approved mutation authority".to_string(),
        });
    }
    if !context.has_required_credentials {
        required.push(AutonomousCodingAuthorityRequirement {
            reason_code: "credentials_required".to_string(),
            summary:
                "Provider or GitHub credentials must be configured before this job can continue."
                    .to_string(),
            required_input:
                "OPENROUTER_API_KEY, GH_TOKEN, GITHUB_TOKEN, or provider-specific credential"
                    .to_string(),
        });
    }
    required
}

fn issue_intake_verifier_plan(
    repo_path: &Path,
    title: &str,
    body: &str,
    classification: AutonomousCodingIssueIntakeClassification,
    context: IssueIntakeAuthorityContext,
) -> AutonomousCodingVerifierPlan {
    let combined = format!("{title}\n{body}");
    let normalized = combined.to_ascii_lowercase();
    let mut missing_inputs = Vec::new();
    if !context.has_verifier {
        missing_inputs.push(
            "verifier command that proves the acceptance criteria before mutation".to_string(),
        );
    }
    if !context.has_edit_or_provider_authority {
        missing_inputs.push(
            "controlled edit, provider repair adapter, or explicit mutation authority".to_string(),
        );
    }
    if !context.has_required_credentials {
        missing_inputs.push("provider or GitHub credential required for this path".to_string());
    }

    match classification {
        AutonomousCodingIssueIntakeClassification::Unsafe => AutonomousCodingVerifierPlan {
            plan_kind: "blocked_unsafe".to_string(),
            summary: "Unsafe request must be rewritten before Tau can plan verification."
                .to_string(),
            suggested_verifier_commands: Vec::new(),
            missing_inputs: vec![
                "remove protected-branch bypass, force-push, secret, or exfiltration request"
                    .to_string(),
                "provide a safe branch/PR workflow".to_string(),
            ],
            next_action: "Rewrite the issue without unsafe authority requests.".to_string(),
        },
        AutonomousCodingIssueIntakeClassification::TooBroad => AutonomousCodingVerifierPlan {
            plan_kind: "needs_scope".to_string(),
            summary: "Issue is too broad for one verifier-gated autonomous job.".to_string(),
            suggested_verifier_commands: vec![
                "cargo fmt --check".to_string(),
                "cargo test -p <affected-crate> <focused-test>".to_string(),
            ],
            missing_inputs: vec![
                "specific module, crate, or file boundary".to_string(),
                "one acceptance criterion that can be verified in one job".to_string(),
                "focused verifier command".to_string(),
            ],
            next_action: "Split the issue into a bounded task with one verifier plan.".to_string(),
        },
        AutonomousCodingIssueIntakeClassification::Underspecified => AutonomousCodingVerifierPlan {
            plan_kind: "needs_issue_detail".to_string(),
            summary: "Issue lacks enough behavior detail to choose a verifier safely.".to_string(),
            suggested_verifier_commands: vec![
                "cargo fmt --check".to_string(),
                "cargo test -p <affected-crate> <focused-test>".to_string(),
            ],
            missing_inputs: vec![
                "expected behavior".to_string(),
                "current failing behavior or reproduction".to_string(),
                "affected file, command, route, or crate".to_string(),
                "acceptance test or verifier command".to_string(),
            ],
            next_action: "Provide expected/current behavior and an affected surface.".to_string(),
        },
        _ if contains_any(&normalized, &["readme", "docs", "documentation", "guide"]) => {
            let commands = derive_concrete_docs_verifier_commands(title, body);
            let commands = if commands.is_empty() {
                if normalized.contains("readme") {
                    vec![
                        "git diff --check".to_string(),
                        "grep -n <expected-text> README.md".to_string(),
                    ]
                } else {
                    vec![
                        "git diff --check".to_string(),
                        "grep -R -n <expected-text> docs".to_string(),
                    ]
                }
            } else {
                commands
            };
            AutonomousCodingVerifierPlan {
                plan_kind: "docs".to_string(),
                summary: "Docs issue can be verified with text assertions and diff hygiene."
                    .to_string(),
                suggested_verifier_commands: commands,
                missing_inputs,
                next_action: if context.has_verifier && context.has_edit_or_provider_authority {
                    "Run the verifier-gated docs job.".to_string()
                } else {
                    "Provide the missing verifier/edit authority or approve provider repair."
                        .to_string()
                },
            }
        }
        _ if contains_any(
            &normalized,
            &["cli", "command", "flag", "argument", "subcommand"],
        ) && !contains_any(
            &normalized,
            &["test", "panic", "rust", "crate", "compile", "clippy"],
        ) =>
        {
            let derivation = derive_repo_aware_code_verifier_plan(repo_path, title, body);
            let derivation_missing_command = derivation.commands.is_empty();
            let report_missing_derivation_inputs =
                derivation_missing_command && !context.has_verifier;
            let mut commands = derivation.commands;
            let mut plan_missing_inputs = missing_inputs;
            if report_missing_derivation_inputs {
                append_missing_inputs(&mut plan_missing_inputs, derivation.missing_inputs);
            }
            if derivation_missing_command {
                commands = vec![
                    "cargo fmt --check".to_string(),
                    "cargo test -p <affected-crate> <focused-test>".to_string(),
                ];
            }
            if normalized.contains("tau-unified") {
                commands.push("scripts/run/test-tau-unified.sh".to_string());
            }
            AutonomousCodingVerifierPlan {
                plan_kind: "cli".to_string(),
                summary: "CLI issue should be verified with focused CLI tests and shell proof."
                    .to_string(),
                suggested_verifier_commands: commands,
                missing_inputs: plan_missing_inputs,
                next_action: if report_missing_derivation_inputs {
                    repo_aware_missing_verifier_next_action()
                } else {
                    "Provide/approve the focused CLI verifier and mutation authority.".to_string()
                },
            }
        }
        _ if contains_any(
            &normalized,
            &["test", "panic", "rust", "crate", "compile", "clippy"],
        ) =>
        {
            let derivation = derive_repo_aware_code_verifier_plan(repo_path, title, body);
            let derivation_missing_command = derivation.commands.is_empty();
            let report_missing_derivation_inputs =
                derivation_missing_command && !context.has_verifier;
            let mut plan_missing_inputs = missing_inputs;
            if report_missing_derivation_inputs {
                append_missing_inputs(&mut plan_missing_inputs, derivation.missing_inputs);
            }
            let commands = if derivation_missing_command {
                vec![
                    "cargo fmt --check".to_string(),
                    "cargo test -p <affected-crate> <focused-test>".to_string(),
                    "cargo clippy -p <affected-crate> --lib --tests -- -D warnings".to_string(),
                ]
            } else {
                derivation.commands
            };
            AutonomousCodingVerifierPlan {
                plan_kind: "rust".to_string(),
                summary: "Rust issue should be verified with focused tests before broader checks."
                    .to_string(),
                suggested_verifier_commands: commands,
                missing_inputs: plan_missing_inputs,
                next_action: if report_missing_derivation_inputs {
                    repo_aware_missing_verifier_next_action()
                } else {
                    "Provide the affected crate/test target or approve Tau's focused verifier plan."
                        .to_string()
                },
            }
        }
        AutonomousCodingIssueIntakeClassification::Solvable
        | AutonomousCodingIssueIntakeClassification::Ready => AutonomousCodingVerifierPlan {
            plan_kind: "ready".to_string(),
            summary: "Issue has enough supplied authority to enter the coding loop.".to_string(),
            suggested_verifier_commands: Vec::new(),
            missing_inputs,
            next_action: "Run issue-to-merge with supplied verifier and edit/provider authority."
                .to_string(),
        },
        AutonomousCodingIssueIntakeClassification::MissingVerifier
        | AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority
        | AutonomousCodingIssueIntakeClassification::MissingCredentials => {
            AutonomousCodingVerifierPlan {
                plan_kind: "generic_coding".to_string(),
                summary: "Issue appears bounded, but Tau needs an explicit verifier and authority."
                    .to_string(),
                suggested_verifier_commands: vec![
                    "cargo fmt --check".to_string(),
                    "cargo test -p <affected-crate> <focused-test>".to_string(),
                ],
                missing_inputs,
                next_action:
                    "Provide verifier command plus edit/provider authority before mutation."
                        .to_string(),
            }
        }
    }
}

fn append_missing_inputs(missing_inputs: &mut Vec<String>, additions: Vec<String>) {
    for addition in additions {
        if !missing_inputs.iter().any(|input| input == &addition) {
            missing_inputs.push(addition);
        }
    }
}

fn repo_aware_missing_verifier_next_action() -> String {
    "Name an existing Cargo package and exact quoted/backticked test filter, or provide an explicit verifier command plus mutation authority."
        .to_string()
}

fn ensure_pr_ready_bundle(
    state: &mut CodingMissionState,
    record: &AutonomousCodingJobRecord,
    started_unix_ms: u64,
    publication: &AutonomousCodingPrPublicationOptions,
) -> Result<CodingMissionPrReadyBundle> {
    if let Some(bundle) = state.pr_ready_bundle.clone() {
        return Ok(bundle);
    }
    let allow_draft_pr = record.pr_mode == CodingMissionPrMode::Draft;
    state
        .prepare_pr_ready_bundle(CodingMissionPrReadyRequest {
            title: Some(format!(
                "Autonomous coding job: {}",
                first_goal_line(&record.goal)
            )),
            risk_notes: vec![
                "Risk: verifier scope is configured by the autonomous coding job".to_string(),
            ],
            rollback_notes: vec!["Revert the autonomous coding mission commit".to_string()],
            allow_draft_pr,
            github_env: publication.github_env.clone(),
            gh_binary: publication.gh_binary.clone(),
            started_unix_ms,
        })
        .map_err(Into::into)
}

fn first_goal_line(goal: &str) -> String {
    goal.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("coding mission")
        .to_string()
}

fn workspace_status_label(status: CodingWorkspaceCommandStatus) -> &'static str {
    match status {
        CodingWorkspaceCommandStatus::Succeeded => "succeeded",
        CodingWorkspaceCommandStatus::Failed => "failed",
        CodingWorkspaceCommandStatus::Denied => "denied",
    }
}

fn pr_publication_status_label(status: CodingMissionPrPublicationStatus) -> &'static str {
    match status {
        CodingMissionPrPublicationStatus::ManualReady => "manual_ready",
        CodingMissionPrPublicationStatus::DraftCreated => "draft_created",
        CodingMissionPrPublicationStatus::DraftFailed => "draft_failed",
    }
}

fn provider_repair_status_label(status: AutonomousCodingProviderRepairStatus) -> &'static str {
    match status {
        AutonomousCodingProviderRepairStatus::Applied => "applied",
        AutonomousCodingProviderRepairStatus::Rejected => "rejected",
        AutonomousCodingProviderRepairStatus::Failed => "failed",
    }
}

fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    let tmp_path = path.with_extension(format!(
        "tmp.{}.{}",
        std::process::id(),
        NEXT_AUTONOMOUS_CODING_JOB_SEQUENCE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&tmp_path, json.as_bytes())
        .with_context(|| format!("failed to write {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::process::Command;

    use serde_json::Value;
    use tau_agent_core::{coding_mission_state_path, load_coding_mission_state};
    use tempfile::TempDir;

    include!("autonomous_coding_jobs_runtime/tests/repo_aware_verifier.rs");

    #[tokio::test]
    async fn spec_c01_submit_persists_mission_and_background_record_link() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_with_echo_runner();
        let submitted = runtime
            .submit_job(fixture.submit_request(true, Vec::new()))
            .await
            .expect("submit job");

        assert!(autonomous_coding_job_record_path(
            runtime.config().state_dir.as_path(),
            submitted.record.job_id.as_str()
        )
        .exists());
        assert!(coding_mission_state_path(
            runtime.config().state_dir.as_path(),
            submitted.record.mission_id.as_str()
        )
        .exists());
        let background = submitted
            .background_job
            .as_ref()
            .expect("background job linked");
        assert_eq!(
            submitted.record.background_job_id.as_deref(),
            Some(background.job_id.as_str())
        );
        assert_eq!(
            submitted.status.background_job_id.as_deref(),
            Some(background.job_id.as_str())
        );
        assert_eq!(submitted.status.status, AutonomousCodingJobStatus::Queued);
    }

    #[tokio::test]
    async fn spec_c02_run_applies_multi_file_edits_commits_and_reaches_pr_ready() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let submitted = runtime
            .submit_job(fixture.submit_request(false, fixture.controlled_edits()))
            .await
            .expect("submit job");

        let outcome = runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 2_000)
            .expect("run job");

        assert_eq!(outcome.status.status, AutonomousCodingJobStatus::PrReady);
        assert_eq!(outcome.status.phase, CodingMissionPhase::PrReady);
        assert!(outcome
            .status
            .changed_files
            .iter()
            .any(|file| file == "status.txt"));
        assert!(outcome
            .status
            .changed_files
            .iter()
            .any(|file| file == "docs/notes.txt"));
        assert!(
            outcome
                .status
                .pr_ready_command
                .as_deref()
                .unwrap_or_default()
                .contains("gh pr create --draft"),
            "missing actionable PR-ready command: {:?}",
            outcome.status.pr_ready_command
        );

        let state = load_coding_mission_state(
            runtime.config().state_dir.as_path(),
            submitted.record.mission_id.as_str(),
        )
        .expect("mission state");
        assert!(state.pr_ready_bundle.is_some());
        assert_eq!(
            git(fixture.repo.path(), &["rev-parse", "--abbrev-ref", "HEAD"]),
            "codex/autonomous-coding-job-mission-alpha"
        );
    }

    #[tokio::test]
    async fn spec_c03_replay_resumes_checkpoint_after_stale_background_recovery() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_with_echo_runner();
        let submitted = runtime
            .submit_job(fixture.submit_request(true, Vec::new()))
            .await
            .expect("submit job");
        let first = runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 2_000)
            .expect("first run");
        assert_eq!(first.status.phase, CodingMissionPhase::Executing);
        assert_eq!(first.status.status, AutonomousCodingJobStatus::Running);
        assert!(first.status.resume_command.contains("mission resume"));

        mark_background_job_stale_running(
            runtime.config().background_jobs_state_dir.as_path(),
            submitted
                .record
                .background_job_id
                .as_deref()
                .expect("background job id"),
        );
        let recovery = runtime
            .recover_stuck_background_jobs()
            .await
            .expect("recover stale background job");
        assert_eq!(recovery.background_report.recovered, 1);
        assert_eq!(recovery.recovered_jobs.len(), 1);
        assert_eq!(recovery.recovered_jobs[0].recovery_count, 1);

        let replayed = runtime
            .replay_job(AutonomousCodingJobReplayRequest {
                job_id: submitted.record.job_id.clone(),
                controlled_edits: fixture.controlled_edits(),
                commit_message: Some("Replay recovered mission to PR ready".to_string()),
                started_unix_ms: 3_000,
            })
            .expect("replay recovered job");

        assert_eq!(replayed.status.status, AutonomousCodingJobStatus::PrReady);
        assert_eq!(replayed.status.recovery_count, 1);
        assert!(replayed.status.replay_count >= 1);
        assert!(
            replayed.status.verifier_summary.contains("succeeded")
                || replayed.status.verifier_summary.contains("failed")
        );
    }

    #[tokio::test]
    async fn spec_c04_status_exposes_operator_action_fields() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let submitted = runtime
            .submit_job(fixture.submit_request(false, fixture.controlled_edits()))
            .await
            .expect("submit job");
        runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 2_000)
            .expect("run job");

        let status = runtime
            .status(submitted.record.job_id.as_str())
            .expect("status");
        assert_eq!(status.job_id, submitted.record.job_id);
        assert_eq!(status.mission_id, submitted.record.mission_id);
        assert_eq!(status.phase, CodingMissionPhase::PrReady);
        assert_ne!(status.verifier_summary, "none");
        assert!(!status.changed_files.is_empty());
        assert_ne!(status.resume_command, "none");
        assert_eq!(status.pr_state, "manual_ready");
        assert!(status.pr_ready_command.is_some());
        assert_eq!(status.operator_state, "complete");
        assert_eq!(status.operator_next_command, "none");
        assert!(!status.replay_safe);
        assert!(!status.recoverable);
        assert!(!status.needs_authority);
        assert!(!status.stale_lease);
        assert!(status
            .mark_blocked_command
            .contains("tau-autonomous-coding-job mark-blocked"));
    }

    #[tokio::test]
    async fn spec_3804_status_classifies_stale_lease_and_marks_blocked() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let submitted = runtime
            .submit_job(fixture.submit_request(false, fixture.controlled_edits()))
            .await
            .expect("submit job");

        let mut record = load_autonomous_coding_job_record(
            runtime.config().state_dir.as_path(),
            submitted.record.job_id.as_str(),
        )
        .expect("load record");
        record.status = AutonomousCodingJobStatus::Running;
        record.reason_code = AUTONOMOUS_CODING_JOB_REASON_RUNNING.to_string();
        record.lease_expires_unix_ms = Some(1);
        persist_autonomous_coding_job_record(&record).expect("persist stale record");

        let status = runtime
            .status(submitted.record.job_id.as_str())
            .expect("stale status");
        assert_eq!(status.operator_state, "stale_lease");
        assert!(status.stale_lease);
        assert!(status.recoverable);
        assert!(status
            .operator_next_command
            .contains("tau-autonomous-coding-job recover"));
        assert!(status
            .mark_blocked_command
            .contains("tau-autonomous-coding-job mark-blocked"));

        let blocked = runtime
            .mark_job_blocked(AutonomousCodingJobMarkBlockedRequest {
                job_id: submitted.record.job_id.clone(),
                reason_code: "operator_marked_blocked".to_string(),
                detail: "operator stopped stale job after inspection".to_string(),
                started_unix_ms: 6_000,
            })
            .expect("mark blocked");
        assert_eq!(blocked.status.status, AutonomousCodingJobStatus::Blocked);
        assert_eq!(blocked.status.operator_state, "blocked");
        assert_eq!(blocked.status.reason_code, "operator_marked_blocked");
        assert!(!blocked.status.stale_lease);
        assert_eq!(
            blocked.status.last_error.as_deref(),
            Some("operator stopped stale job after inspection")
        );
    }

    #[tokio::test]
    async fn spec_3796_c01_auto_merge_requests_gh_auto_merge_when_authorized() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let submitted = make_pr_ready_job_with_pr_url(&fixture, &runtime).await;
        let gh = fixture.fake_gh("auto-merge-enabled\n", 0);
        let outcome = runtime
            .request_auto_merge(AutonomousCodingAutoMergeRequest {
                job_id: submitted.record.job_id.clone(),
                allow_auto_merge: true,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: true,
                github_env: BTreeMap::from([("GH_TOKEN".to_string(), "test-token".to_string())]),
                gh_binary: Some(gh.binary.clone()),
                started_unix_ms: 4_000,
            })
            .expect("request auto merge");

        assert_eq!(
            outcome.evidence.status,
            AutonomousCodingAutoMergeStatus::Requested
        );
        assert_eq!(
            outcome.status.auto_merge_status.as_deref(),
            Some("requested")
        );
        assert_eq!(
            outcome.status.auto_merge_reason_code.as_deref(),
            Some("auto_merge_requested")
        );
        let argv = std::fs::read_to_string(&gh.argv_path).expect("captured argv");
        assert!(argv.contains("pr\nmerge\nhttps://github.com/njfio/Tau/pull/3796"));
        assert!(argv.contains("--auto\n"));
        assert!(argv.contains("--squash\n"));
        assert!(argv.contains("--delete-branch\n"));
        assert!(!argv.contains("--admin"));
    }

    #[tokio::test]
    async fn spec_3796_c02_auto_merge_blocks_without_policy_or_pr_url() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let submitted = runtime
            .submit_job(fixture.submit_request(false, fixture.controlled_edits()))
            .await
            .expect("submit job");
        runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 2_000)
            .expect("run job");
        let gh = fixture.fake_gh("should-not-run\n", 0);

        let outcome = runtime
            .request_auto_merge(AutonomousCodingAutoMergeRequest {
                job_id: submitted.record.job_id.clone(),
                allow_auto_merge: false,
                merge_method: AutonomousCodingMergeMethod::Merge,
                delete_branch: false,
                github_env: BTreeMap::from([("GH_TOKEN".to_string(), "test-token".to_string())]),
                gh_binary: Some(gh.binary.clone()),
                started_unix_ms: 4_000,
            })
            .expect("blocked auto merge");

        assert_eq!(
            outcome.evidence.status,
            AutonomousCodingAutoMergeStatus::Blocked
        );
        assert_eq!(outcome.evidence.reason_code, "auto_merge_policy_disabled");
        assert!(!gh.argv_path.exists(), "blocked merge should not invoke gh");
    }

    #[tokio::test]
    async fn spec_3796_c04_issue_intake_without_authority_persists_blocked_plan() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let before_status =
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status before");

        let outcome = runtime
            .intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                intake_id: "issue-3796-intake".to_string(),
                issue_url: "https://github.com/njfio/Tau/issues/3796".to_string(),
                issue_title: "Solve arbitrary issue".to_string(),
                issue_body: "Make Tau solve this without verifier/edit authority.".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                base_branch: "master".to_string(),
                started_unix_ms: 5_000,
            })
            .expect("issue intake");

        assert_eq!(outcome.status, AutonomousCodingIssueIntakeStatus::Blocked);
        assert_eq!(
            outcome.classification,
            AutonomousCodingIssueIntakeClassification::MissingVerifier
        );
        assert_eq!(outcome.reason_code, "issue_intake_authority_required");
        assert!(outcome
            .required_authority
            .iter()
            .any(|item| item.reason_code == "verifier_authority_required"));
        assert!(outcome
            .required_authority
            .iter()
            .any(|item| item.reason_code == "edit_authority_required"));
        assert_eq!(outcome.verifier_plan.plan_kind, "generic_coding");
        assert!(outcome
            .verifier_plan
            .suggested_verifier_commands
            .iter()
            .any(|command| command.contains("cargo test")));
        assert!(outcome
            .missing_inputs
            .iter()
            .any(|input| input.contains("verifier command")));
        assert!(outcome
            .next_action_summary
            .contains("Provide verifier command"));
        assert!(autonomous_coding_issue_intake_path(
            runtime.config().state_dir.as_path(),
            outcome.intake_id.as_str()
        )
        .exists());
        assert_eq!(
            outcome.issue_body,
            "Make Tau solve this without verifier/edit authority."
        );
        let persisted = runtime
            .issue_intake_status(outcome.intake_id.as_str())
            .expect("persisted intake status");
        assert_eq!(persisted.issue_body, outcome.issue_body);
        assert_eq!(
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status after"),
            before_status
        );

        let unsafe_outcome = runtime
            .intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                intake_id: "issue-3796-unsafe".to_string(),
                issue_url: "https://github.com/njfio/Tau/issues/3796".to_string(),
                issue_title: "Bypass branch protection".to_string(),
                issue_body: "Use --admin to bypass branch protection and merge anyway.".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                base_branch: "master".to_string(),
                started_unix_ms: 5_001,
            })
            .expect("unsafe issue intake");
        assert_eq!(
            unsafe_outcome.classification,
            AutonomousCodingIssueIntakeClassification::Unsafe
        );
        assert_eq!(unsafe_outcome.reason_code, "issue_intake_unsafe");
        assert_eq!(unsafe_outcome.verifier_plan.plan_kind, "blocked_unsafe");
    }

    #[tokio::test]
    async fn spec_3806_c03_c04_issue_intake_generates_verifier_plan() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();

        let docs = runtime
            .intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                intake_id: "issue-3806-docs".to_string(),
                issue_url: "https://github.com/njfio/Tau/issues/3806".to_string(),
                issue_title: "Update README docs".to_string(),
                issue_body: "Document the operator recovery workflow in the README and guide."
                    .to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                base_branch: "master".to_string(),
                started_unix_ms: 5_100,
            })
            .expect("docs intake");
        assert_eq!(
            docs.classification,
            AutonomousCodingIssueIntakeClassification::MissingVerifier
        );
        assert_eq!(docs.verifier_plan.plan_kind, "docs");
        assert!(docs
            .verifier_plan
            .suggested_verifier_commands
            .iter()
            .any(|command| command == "git diff --check"));
        assert!(docs
            .missing_inputs
            .iter()
            .any(|input| input.contains("provider")));

        let broad = runtime
            .intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                intake_id: "issue-3806-broad".to_string(),
                issue_url: "https://github.com/njfio/Tau/issues/3806".to_string(),
                issue_title: "Fix everything".to_string(),
                issue_body: "Make the entire repo fully autonomous forever with any issue."
                    .to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                base_branch: "master".to_string(),
                started_unix_ms: 5_101,
            })
            .expect("broad intake");
        assert_eq!(
            broad.classification,
            AutonomousCodingIssueIntakeClassification::TooBroad
        );
        assert_eq!(broad.verifier_plan.plan_kind, "needs_scope");
        assert!(broad
            .verifier_plan
            .missing_inputs
            .iter()
            .any(|input| input.contains("specific module")));
        assert!(broad.next_action_summary.contains("Split the issue"));
    }

    #[tokio::test]
    async fn spec_3807_c01_underspecified_intake_persists_clarifying_questions() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();

        let outcome = runtime
            .intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                intake_id: "issue-3807-vague".to_string(),
                issue_url: "https://github.com/njfio/Tau/issues/3807".to_string(),
                issue_title: "Fix it".to_string(),
                issue_body: "Broken".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                base_branch: "master".to_string(),
                started_unix_ms: 5_200,
            })
            .expect("vague intake");

        assert_eq!(
            outcome.classification,
            AutonomousCodingIssueIntakeClassification::Underspecified
        );
        assert_eq!(
            outcome.decision,
            AutonomousCodingIssueIntakeDecision::NeedsClarification
        );
        let reason_codes: Vec<_> = outcome
            .clarifying_questions
            .iter()
            .map(|question| question.reason_code.as_str())
            .collect();
        assert!(reason_codes.contains(&"expected_behavior"));
        assert!(reason_codes.contains(&"current_behavior"));
        assert!(reason_codes.contains(&"affected_surface"));
        assert!(reason_codes.contains(&"verifier_command"));

        let persisted = runtime
            .issue_intake_status("issue-3807-vague")
            .expect("persisted vague intake");
        assert_eq!(persisted.decision, outcome.decision);
        assert_eq!(persisted.clarifying_questions, outcome.clarifying_questions);
    }

    #[tokio::test]
    async fn spec_3807_c02_broad_intake_asks_for_split_contract() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();

        let outcome = runtime
            .intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                intake_id: "issue-3807-broad".to_string(),
                issue_url: "https://github.com/njfio/Tau/issues/3807".to_string(),
                issue_title: "Make Tau solve any issue".to_string(),
                issue_body: "Make the entire repo handle every arbitrary issue automatically."
                    .to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                base_branch: "master".to_string(),
                started_unix_ms: 5_201,
            })
            .expect("broad intake");

        assert_eq!(
            outcome.classification,
            AutonomousCodingIssueIntakeClassification::TooBroad
        );
        assert_eq!(
            outcome.decision,
            AutonomousCodingIssueIntakeDecision::SplitRequired
        );
        let reason_codes: Vec<_> = outcome
            .clarifying_questions
            .iter()
            .map(|question| question.reason_code.as_str())
            .collect();
        assert!(reason_codes.contains(&"bounded_surface"));
        assert!(reason_codes.contains(&"single_acceptance_criterion"));
    }

    #[tokio::test]
    async fn spec_3807_c03_ready_intake_with_authority_is_machine_readable() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();

        let outcome = runtime
            .intake_issue_with_context(
                AutonomousCodingIssueIntakeRequest {
                    intake_id: "issue-3807-ready".to_string(),
                    issue_url: "https://github.com/njfio/Tau/issues/3807".to_string(),
                    issue_title: "Fix tau-runtime panic in coding job replay".to_string(),
                    issue_body:
                        "Replay fails when a persisted checkpoint exists; verify with the focused tau-runtime replay test."
                            .to_string(),
                    repo_path: fixture.repo.path().to_path_buf(),
                    base_branch: "master".to_string(),
                    started_unix_ms: 5_202,
                },
                IssueIntakeAuthorityContext {
                    has_verifier: true,
                    has_edit_or_provider_authority: true,
                    has_required_credentials: true,
                },
            )
            .expect("ready intake");

        assert_eq!(
            outcome.classification,
            AutonomousCodingIssueIntakeClassification::Solvable
        );
        assert_eq!(
            outcome.decision,
            AutonomousCodingIssueIntakeDecision::ReadyToRun
        );
        assert!(outcome.clarifying_questions.is_empty());
        assert!(outcome.missing_inputs.is_empty());
        assert_eq!(
            outcome.issue_body,
            "Replay fails when a persisted checkpoint exists; verify with the focused tau-runtime replay test."
        );
    }

    #[test]
    fn spec_3807_c04_legacy_intake_json_defaults_new_fields() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let path = autonomous_coding_issue_intake_path(
            runtime.config().state_dir.as_path(),
            "issue-3807-legacy",
        );
        std::fs::create_dir_all(path.parent().expect("intake parent")).expect("intake dir");
        std::fs::write(
            &path,
            serde_json::json!({
                "schema_version": AUTONOMOUS_CODING_JOB_SCHEMA_VERSION,
                "intake_id": "issue-3807-legacy",
                "status": "blocked",
                "reason_code": "issue_intake_authority_required",
                "classification": "missing_verifier",
                "classification_summary": "legacy payload",
                "issue_url": "https://github.com/njfio/Tau/issues/3807",
                "issue_title": "Legacy intake",
                "issue_body_summary": "legacy",
                "repo_path": fixture.repo.path(),
                "base_branch": "master",
                "required_authority": [],
                "verifier_plan": {
                    "plan_kind": "generic_coding",
                    "summary": "legacy",
                    "suggested_verifier_commands": [],
                    "missing_inputs": [],
                    "next_action": "legacy"
                },
                "missing_inputs": [],
                "next_action_summary": "legacy",
                "created_unix_ms": 1,
                "updated_unix_ms": 1
            })
            .to_string(),
        )
        .expect("write legacy intake");

        let loaded = runtime
            .issue_intake_status("issue-3807-legacy")
            .expect("load legacy intake");
        assert_eq!(
            loaded.decision,
            AutonomousCodingIssueIntakeDecision::Unknown
        );
        assert!(loaded.clarifying_questions.is_empty());
        assert_eq!(loaded.issue_body, "");
    }

    #[tokio::test]
    async fn spec_3801_c01_issue_to_merge_runs_pr_ready_and_requests_auto_merge() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let gh = fixture.fake_gh("https://github.com/njfio/Tau/pull/3801\n", 0);

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3801-intake".to_string(),
                mission_id: "issue-3801-mission".to_string(),
                session_key: "issue-3801-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3801".to_string(),
                issue_title: "Make issue-to-merge hands-off".to_string(),
                issue_body: "Run the authorized coding loop through merge request.".to_string(),
                goal: "Make the verifier pass and request auto-merge".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-to-merge-".to_string(),
                verifier_commands: vec![
                    "grep -q pass status.txt".to_string(),
                    "grep -q proof docs/notes.txt".to_string(),
                ],
                pr_mode: CodingMissionPrMode::Draft,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: fixture.controlled_edits(),
                provider_repair: AutonomousCodingProviderRepairPolicy::default(),
                commit_message: "Make issue-to-merge verifier green".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: true,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: true,
                github_env: BTreeMap::from([("GH_TOKEN".to_string(), "test-token".to_string())]),
                gh_binary: Some(gh.binary.clone()),
                started_unix_ms: 6_000,
            })
            .await
            .expect("issue to merge");

        assert_eq!(
            outcome.status,
            AutonomousCodingIssueToMergeStatus::AutoMergeRequested
        );
        assert_eq!(outcome.reason_code, "auto_merge_requested");
        assert!(outcome.intake.is_none());
        let run = outcome.run.as_ref().expect("run outcome");
        assert_eq!(run.status.status, AutonomousCodingJobStatus::PrReady);
        assert_eq!(run.status.pr_state, "draft_created");
        assert_eq!(
            run.status.pr_url.as_deref(),
            Some("https://github.com/njfio/Tau/pull/3801")
        );
        let auto_merge = outcome.auto_merge.as_ref().expect("auto merge");
        assert_eq!(
            auto_merge.evidence.status,
            AutonomousCodingAutoMergeStatus::Requested
        );
        assert_eq!(
            auto_merge.evidence.pr_url.as_deref(),
            Some("https://github.com/njfio/Tau/pull/3801")
        );

        let argv = std::fs::read_to_string(&gh.argv_path).expect("captured argv");
        assert!(argv.contains("pr\nmerge\nhttps://github.com/njfio/Tau/pull/3801"));
        assert!(argv.contains("--auto\n"));
        assert!(argv.contains("--squash\n"));
        assert!(argv.contains("--delete-branch\n"));
        assert!(!argv.contains("--admin"));
    }

    #[tokio::test]
    async fn spec_3806_c05_draft_pr_creation_records_github_evidence() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let gh = fixture.fake_gh_create_or_update(None, "https://github.com/njfio/Tau/pull/3806");

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3806-draft-create".to_string(),
                mission_id: "issue-3806-draft-create-mission".to_string(),
                session_key: "issue-3806-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3806".to_string(),
                issue_title: "Create draft PR by default".to_string(),
                issue_body: "Run the authorized loop and publish a draft PR.".to_string(),
                goal: "Make verifier pass and create draft PR".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-3806-draft-".to_string(),
                verifier_commands: vec!["grep -q pass status.txt".to_string()],
                pr_mode: CodingMissionPrMode::Draft,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: vec![CodingMissionControlledEdit {
                    relative_path: PathBuf::from("status.txt"),
                    contents: "pass\n".to_string(),
                    reason_code: "controlled_status_fix".to_string(),
                }],
                provider_repair: AutonomousCodingProviderRepairPolicy::default(),
                commit_message: "Make draft PR verifier green".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: false,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: false,
                github_env: BTreeMap::from([("GH_TOKEN".to_string(), "test-token".to_string())]),
                gh_binary: Some(gh.binary.clone()),
                started_unix_ms: 6_500,
            })
            .await
            .expect("issue to merge draft create");

        assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::PrReady);
        let run = outcome.run.as_ref().expect("run");
        assert_eq!(run.status.pr_state, "draft_created");
        assert_eq!(
            run.status.pr_url.as_deref(),
            Some("https://github.com/njfio/Tau/pull/3806")
        );
        assert_eq!(
            run.status.pr_publication_reason_code.as_deref(),
            Some("draft_pr_created")
        );
        assert!(run
            .status
            .pr_publication_command
            .as_deref()
            .is_some_and(|command| command.contains("pr create")));
        assert!(run.status.pr_publication_stdout_path.is_some());

        let argv = std::fs::read_to_string(&gh.argv_path).expect("captured gh argv");
        assert!(argv.contains("pr\nlist\n"));
        assert!(argv.contains("pr\ncreate\n"));
        assert!(argv.contains("--draft\n"));
        assert!(!argv.contains("--admin"));
    }

    #[tokio::test]
    async fn spec_3806_c06_missing_github_auth_keeps_manual_draft_command() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3806-draft-manual".to_string(),
                mission_id: "issue-3806-draft-manual-mission".to_string(),
                session_key: "issue-3806-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3806".to_string(),
                issue_title: "Manual draft PR fallback".to_string(),
                issue_body: "Run the authorized loop without GitHub credentials.".to_string(),
                goal: "Make verifier pass and produce manual draft PR command".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-3806-manual-".to_string(),
                verifier_commands: vec!["grep -q pass status.txt".to_string()],
                pr_mode: CodingMissionPrMode::Draft,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: vec![CodingMissionControlledEdit {
                    relative_path: PathBuf::from("status.txt"),
                    contents: "pass\n".to_string(),
                    reason_code: "controlled_status_fix".to_string(),
                }],
                provider_repair: AutonomousCodingProviderRepairPolicy::default(),
                commit_message: "Make manual draft verifier green".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: false,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: false,
                github_env: BTreeMap::new(),
                gh_binary: None,
                started_unix_ms: 6_600,
            })
            .await
            .expect("issue to merge manual draft");

        assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::PrReady);
        let run = outcome.run.as_ref().expect("run");
        assert_eq!(run.status.pr_state, "manual_ready");
        assert_eq!(
            run.status.pr_publication_reason_code.as_deref(),
            Some("draft_pr_missing_github_auth")
        );
        assert!(run
            .status
            .pr_ready_command
            .as_deref()
            .is_some_and(|command| command.contains("gh pr create --draft")));
        assert!(run.status.pr_publication_command.is_none());
    }

    #[tokio::test]
    async fn spec_3801_c03_issue_to_merge_without_edit_authority_blocks_before_mutation() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let before_status =
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status before");

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3801-no-authority".to_string(),
                mission_id: "issue-3801-no-authority-mission".to_string(),
                session_key: "issue-3801-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3801".to_string(),
                issue_title: "Missing edit authority".to_string(),
                issue_body: "No edit plan has been granted.".to_string(),
                goal: "Should not mutate without edit authority".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-to-merge-".to_string(),
                verifier_commands: vec!["grep -q pass status.txt".to_string()],
                pr_mode: CodingMissionPrMode::PrReady,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: Vec::new(),
                provider_repair: AutonomousCodingProviderRepairPolicy::default(),
                commit_message: "Should not be used".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: true,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: true,
                github_env: BTreeMap::from([("GH_TOKEN".to_string(), "test-token".to_string())]),
                gh_binary: None,
                started_unix_ms: 6_000,
            })
            .await
            .expect("issue to merge blocked");

        assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::Blocked);
        assert_eq!(outcome.reason_code, "issue_intake_authority_required");
        let intake = outcome.intake.as_ref().expect("intake");
        assert_eq!(
            intake.classification,
            AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority
        );
        assert!(intake
            .required_authority
            .iter()
            .any(|item| item.reason_code == "edit_authority_required"));
        assert!(!intake
            .required_authority
            .iter()
            .any(|item| item.reason_code == "verifier_authority_required"));
        assert!(outcome.submit.is_none());
        assert!(outcome.run.is_none());
        assert!(outcome.auto_merge.is_none());
        assert_eq!(
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status after"),
            before_status
        );
        let jobs_dir = runtime.config().state_dir.join("autonomous-coding-jobs");
        let job_count = std::fs::read_dir(jobs_dir)
            .expect("jobs dir")
            .filter_map(Result::ok)
            .count();
        assert_eq!(job_count, 0, "blocked intake should not create jobs");
    }

    #[tokio::test]
    async fn spec_3802_c01_provider_repair_runs_inside_durable_job_loop() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let provider = fixture.fake_provider_repair(
            r#"{"edits":[{"relative_path":"status.txt","contents":"pass\n","reason_code":"provider_status_fix"},{"relative_path":"docs/notes.txt","contents":"proof\n","reason_code":"provider_notes_fix"}]}"#,
            0,
        );
        let mut request = fixture.submit_request(false, Vec::new());
        request.provider_repair = provider.policy(2, "fake-provider", "repair-model");

        let submitted = runtime.submit_job(request).await.expect("submit job");
        let outcome = runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 8_000)
            .expect("run with provider repair");

        assert_eq!(outcome.status.status, AutonomousCodingJobStatus::PrReady);
        assert_eq!(outcome.status.provider_repair_attempts, 1);
        assert_eq!(
            outcome.status.provider_repair_status.as_deref(),
            Some("applied")
        );
        assert_eq!(
            outcome.status.provider_repair_reason_code.as_deref(),
            Some("provider_repair_edit_parsed")
        );
        assert_eq!(
            outcome.status.provider_repair_provider.as_deref(),
            Some("fake-provider")
        );
        assert_eq!(
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status"),
            "pass\n"
        );
        assert_eq!(
            std::fs::read_to_string(fixture.repo.path().join("docs/notes.txt")).expect("notes"),
            "proof\n"
        );
        assert!(outcome.status.event_log_path.exists());
        let events = std::fs::read_to_string(&outcome.status.event_log_path).expect("events");
        assert!(events.contains("provider_repair_requested"));
        assert!(events.contains("provider_repair_ready"));
        let context_path =
            std::fs::read_to_string(&provider.context_path_capture).expect("context path");
        let context_raw = std::fs::read_to_string(context_path.trim()).expect("context");
        let context: Value = serde_json::from_str(&context_raw).expect("context json");
        assert_eq!(context["failed_verifiers"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn spec_3802_c02_provider_repair_accepts_unified_diff() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let provider = fixture.fake_provider_repair(
            r#"{"diff":"--- a/status.txt\n+++ b/status.txt\n@@ -1 +1 @@\n-fail\n+pass\n","reason_code":"provider_diff_fix"}"#,
            0,
        );
        let mut request = fixture.submit_request(false, Vec::new());
        request.verifier_commands = vec!["grep -q pass status.txt".to_string()];
        request.provider_repair = provider.policy(1, "fake-provider", "diff-model");

        let submitted = runtime.submit_job(request).await.expect("submit job");
        let outcome = runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 9_000)
            .expect("run with provider diff");

        assert_eq!(outcome.status.status, AutonomousCodingJobStatus::PrReady);
        assert_eq!(outcome.status.provider_repair_attempts, 1);
        assert_eq!(outcome.status.changed_files, vec!["status.txt"]);
        assert_eq!(
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status"),
            "pass\n"
        );
    }

    #[tokio::test]
    async fn spec_3802_c03_malformed_provider_repair_blocks_without_commit() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let provider = fixture.fake_provider_repair("not-json", 0);
        let mut request = fixture.submit_request(false, Vec::new());
        request.provider_repair = provider.policy(2, "fake-provider", "bad-model");

        let submitted = runtime.submit_job(request).await.expect("submit job");
        let outcome = runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 10_000)
            .expect("run with malformed provider repair");

        assert_eq!(outcome.status.status, AutonomousCodingJobStatus::Blocked);
        assert_eq!(
            outcome.status.reason_code,
            AUTONOMOUS_CODING_JOB_REASON_PROVIDER_REPAIR_EXHAUSTED
        );
        assert_eq!(outcome.status.provider_repair_attempts, 2);
        assert_eq!(
            outcome.status.provider_repair_status.as_deref(),
            Some("rejected")
        );
        assert_eq!(
            outcome.status.provider_repair_reason_code.as_deref(),
            Some("provider_repair_output_invalid")
        );
        assert_eq!(
            std::fs::read_to_string(fixture.repo.path().join("status.txt")).expect("status"),
            "fail\n"
        );
        let log = git(fixture.repo.path(), &["log", "--oneline"]);
        assert_eq!(log.lines().count(), 1, "blocked repair should not commit");
    }

    #[tokio::test]
    async fn spec_3802_c04_issue_to_merge_uses_provider_repair_without_manual_edits() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let provider = fixture.fake_provider_repair(
            r#"{"files":{"status.txt":"pass\n","docs/notes.txt":"proof\n"},"reason_code":"provider_file_map_fix"}"#,
            0,
        );

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3802-provider-intake".to_string(),
                mission_id: "issue-3802-provider-mission".to_string(),
                session_key: "issue-3802-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3802".to_string(),
                issue_title: "Use provider repair".to_string(),
                issue_body: "Run verifier repair without manual edit authority.".to_string(),
                goal: "Make the verifier pass through provider repair".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-to-merge-provider-".to_string(),
                verifier_commands: vec![
                    "grep -q pass status.txt".to_string(),
                    "grep -q proof docs/notes.txt".to_string(),
                ],
                pr_mode: CodingMissionPrMode::PrReady,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: Vec::new(),
                provider_repair: provider.policy(2, "fake-provider", "repair-model"),
                commit_message: "Repair via provider".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: false,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: false,
                github_env: BTreeMap::new(),
                gh_binary: None,
                started_unix_ms: 11_000,
            })
            .await
            .expect("issue-to-merge provider repair");

        assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::PrReady);
        let run = outcome.run.as_ref().expect("run");
        assert_eq!(run.status.status, AutonomousCodingJobStatus::PrReady);
        assert_eq!(run.status.provider_repair_attempts, 1);
        assert!(outcome.intake.is_none());
    }

    #[tokio::test]
    async fn spec_3809_c01_issue_to_merge_derives_concrete_docs_verifier() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let provider = fixture.fake_provider_repair(
            r#"{"files":{"docs/notes.txt":"tau_derived_docs_verifier\n"},"reason_code":"provider_docs_marker"}"#,
            0,
        );

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3809-derived-docs".to_string(),
                mission_id: "issue-3809-derived-docs-mission".to_string(),
                session_key: "issue-3809-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3809".to_string(),
                issue_title: "Document `tau_derived_docs_verifier` in docs".to_string(),
                issue_body:
                    "Add documentation that includes the exact marker `tau_derived_docs_verifier`."
                        .to_string(),
                goal: "Let Tau derive the docs verifier from intake.".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-to-merge-derived-docs-".to_string(),
                verifier_commands: Vec::new(),
                pr_mode: CodingMissionPrMode::PrReady,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: Vec::new(),
                provider_repair: provider.policy(2, "fake-provider", "repair-model"),
                commit_message: "Document derived verifier marker".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: false,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: false,
                github_env: BTreeMap::new(),
                gh_binary: None,
                started_unix_ms: 12_000,
            })
            .await
            .expect("issue-to-merge derived docs verifier");

        assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::PrReady);
        let intake = outcome.intake.as_ref().expect("derived intake");
        assert_eq!(
            intake.decision,
            AutonomousCodingIssueIntakeDecision::ReadyToRun
        );
        assert_eq!(intake.verifier_plan.plan_kind, "docs");
        assert!(intake
            .verifier_plan
            .suggested_verifier_commands
            .iter()
            .any(|command| command == "grep -R -n tau_derived_docs_verifier docs"));

        let submit = outcome.submit.as_ref().expect("submit");
        assert!(submit
            .record
            .verifier_commands
            .iter()
            .any(|command| command == "grep -R -n tau_derived_docs_verifier docs"));
        let run = outcome.run.as_ref().expect("run");
        assert_eq!(run.status.status, AutonomousCodingJobStatus::PrReady);
        assert!(run
            .status
            .changed_files
            .iter()
            .any(|file| file == "docs/notes.txt"));
    }

    #[tokio::test]
    async fn spec_3809_c02_issue_to_merge_without_concrete_docs_verifier_still_blocks() {
        let fixture = CodingJobFixture::new();
        let runtime = fixture.runtime_without_background();
        let provider = fixture.fake_provider_repair(
            r#"{"files":{"docs/notes.txt":"operator recovery evidence\n"},"reason_code":"provider_docs_text"}"#,
            0,
        );

        let outcome = runtime
            .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                intake_id: "issue-3809-missing-docs-verifier".to_string(),
                mission_id: "issue-3809-missing-docs-verifier-mission".to_string(),
                session_key: "issue-3809-session".to_string(),
                repo_path: fixture.repo.path().to_path_buf(),
                issue_url: "https://github.com/njfio/Tau/issues/3809".to_string(),
                issue_title: "Update the docs".to_string(),
                issue_body: "Explain the operator recovery evidence workflow in documentation."
                    .to_string(),
                goal: "Should not mutate without a concrete verifier assertion.".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/issue-to-merge-missing-docs-verifier-".to_string(),
                verifier_commands: Vec::new(),
                pr_mode: CodingMissionPrMode::PrReady,
                allowed_roots: vec![fixture.repo.path().to_path_buf()],
                controlled_edits: Vec::new(),
                provider_repair: provider.policy(2, "fake-provider", "repair-model"),
                commit_message: "Should not be used".to_string(),
                timeout_ms: Some(5_000),
                allow_auto_merge: false,
                merge_method: AutonomousCodingMergeMethod::Squash,
                delete_branch: false,
                github_env: BTreeMap::new(),
                gh_binary: None,
                started_unix_ms: 13_000,
            })
            .await
            .expect("issue-to-merge blocks without concrete docs verifier");

        assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::Blocked);
        let intake = outcome.intake.as_ref().expect("blocked intake");
        assert_eq!(
            intake.classification,
            AutonomousCodingIssueIntakeClassification::MissingVerifier
        );
        assert!(intake
            .required_authority
            .iter()
            .any(|authority| authority.reason_code == "verifier_authority_required"));
        assert!(outcome.submit.is_none());
        assert!(outcome.run.is_none());
    }

    struct CodingJobFixture {
        root: TempDir,
        repo: TempDir,
    }

    impl CodingJobFixture {
        fn new() -> Self {
            let root = TempDir::new().expect("state root");
            let repo = TempDir::new().expect("repo root");
            git(repo.path(), &["init", "-b", "master"]);
            git(repo.path(), &["config", "user.email", "tau@example.test"]);
            git(repo.path(), &["config", "user.name", "Tau Test"]);
            std::fs::write(repo.path().join("status.txt"), "fail\n").expect("status");
            std::fs::create_dir_all(repo.path().join("docs")).expect("docs");
            std::fs::write(repo.path().join("docs/notes.txt"), "draft\n").expect("notes");
            git(repo.path(), &["add", "."]);
            git(repo.path(), &["commit", "-m", "Initial fixture"]);
            Self { root, repo }
        }

        fn runtime_without_background(&self) -> AutonomousCodingJobRuntime {
            AutonomousCodingJobRuntime::new(AutonomousCodingJobRuntimeConfig {
                state_dir: self.root.path().join("autonomous"),
                background_jobs_state_dir: self.root.path().join("jobs"),
                runner_command: PathBuf::from("/bin/echo"),
                runner_args_prefix: Vec::new(),
                default_timeout_ms: 5_000,
                max_timeout_ms: 10_000,
                worker_poll_ms: 10,
                stuck_recovery_poll_ms: 0,
            })
            .expect("runtime")
        }

        fn runtime_with_echo_runner(&self) -> AutonomousCodingJobRuntime {
            AutonomousCodingJobRuntime::new(AutonomousCodingJobRuntimeConfig {
                state_dir: self.root.path().join("autonomous"),
                background_jobs_state_dir: self.root.path().join("jobs"),
                runner_command: PathBuf::from("/bin/echo"),
                runner_args_prefix: Vec::new(),
                default_timeout_ms: 5_000,
                max_timeout_ms: 10_000,
                worker_poll_ms: 10,
                stuck_recovery_poll_ms: 0,
            })
            .expect("runtime")
        }

        fn submit_request(
            &self,
            enqueue_background_job: bool,
            controlled_edits: Vec<CodingMissionControlledEdit>,
        ) -> AutonomousCodingJobSubmitRequest {
            AutonomousCodingJobSubmitRequest {
                mission_id: "mission-alpha".to_string(),
                session_key: "session-alpha".to_string(),
                repo_path: self.repo.path().to_path_buf(),
                issue_url: Some("https://github.com/njfio/Tau/issues/3794".to_string()),
                goal: "Make the verifier pass and prepare PR-ready evidence".to_string(),
                base_branch: "master".to_string(),
                branch_prefix: "codex/autonomous-coding-job-".to_string(),
                verifier_commands: vec![
                    "grep -q pass status.txt".to_string(),
                    "grep -q proof docs/notes.txt".to_string(),
                ],
                pr_mode: CodingMissionPrMode::PrReady,
                allowed_roots: vec![self.repo.path().to_path_buf()],
                controlled_edits,
                provider_repair: AutonomousCodingProviderRepairPolicy::default(),
                commit_message: "Make autonomous verifier green".to_string(),
                enqueue_background_job,
                timeout_ms: Some(5_000),
                started_unix_ms: 1_000,
            }
        }

        fn controlled_edits(&self) -> Vec<CodingMissionControlledEdit> {
            vec![
                CodingMissionControlledEdit {
                    relative_path: PathBuf::from("status.txt"),
                    contents: "pass\n".to_string(),
                    reason_code: "controlled_status_fix".to_string(),
                },
                CodingMissionControlledEdit {
                    relative_path: PathBuf::from("docs/notes.txt"),
                    contents: "proof\n".to_string(),
                    reason_code: "controlled_notes_fix".to_string(),
                },
            ]
        }

        fn install_rust_workspace_fixture(&self, marker: &str) {
            std::fs::create_dir_all(self.repo.path().join("crates/fixture-cli/src"))
                .expect("fixture crate dir");
            std::fs::write(
                self.repo.path().join("Cargo.toml"),
                "[workspace]\nmembers = [\"crates/fixture-cli\"]\nresolver = \"2\"\n",
            )
            .expect("workspace cargo toml");
            std::fs::write(
                self.repo.path().join("crates/fixture-cli/Cargo.toml"),
                "[package]\nname = \"fixture-cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            )
            .expect("fixture cargo toml");
            std::fs::write(
                self.repo.path().join("crates/fixture-cli/src/lib.rs"),
                format!(
                    "pub fn marker() -> &'static str {{\n    \"{marker}\"\n}}\n\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn spec_3810_repo_aware_verifier() {{\n        assert_eq!(marker(), \"pass\");\n    }}\n}}\n"
                ),
            )
            .expect("fixture lib");
            git(self.repo.path(), &["add", "."]);
            git(
                self.repo.path(),
                &["commit", "-m", "Add Rust workspace fixture"],
            );
        }

        fn fake_gh(&self, stdout: &str, exit_code: i32) -> FakeGh {
            let binary = self.root.path().join("fake-gh.sh");
            let argv_path = self.root.path().join("fake-gh-argv.txt");
            let script = format!(
                "#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{}'\nprintf '{}'\nexit {}\n",
                shell_single_quote(argv_path.display().to_string().as_str()),
                stdout.replace('\'', "'\"'\"'"),
                exit_code
            );
            std::fs::write(&binary, script).expect("fake gh");
            let mut perms = std::fs::metadata(&binary).expect("metadata").permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                perms.set_mode(0o755);
                std::fs::set_permissions(&binary, perms).expect("chmod");
            }
            FakeGh { binary, argv_path }
        }

        fn fake_gh_create_or_update(
            &self,
            existing_pr_url: Option<&str>,
            created_pr_url: &str,
        ) -> FakeGh {
            let binary = self.root.path().join("fake-gh-create-or-update.sh");
            let argv_path = self.root.path().join("fake-gh-create-or-update-argv.txt");
            let existing = existing_pr_url.unwrap_or("");
            let script = format!(
                "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' \"$@\" >> '{}'\nif [[ \"${{1:-}} ${{2:-}}\" == \"pr list\" ]]; then\n  printf '{}\\n'\n  exit 0\nfi\nif [[ \"${{1:-}} ${{2:-}}\" == \"pr create\" ]]; then\n  printf '{}\\n'\n  exit 0\nfi\nif [[ \"${{1:-}} ${{2:-}}\" == \"pr edit\" ]]; then\n  exit 0\nfi\necho unexpected gh command >&2\nexit 12\n",
                shell_single_quote(argv_path.display().to_string().as_str()),
                existing.replace('\'', "'\"'\"'"),
                created_pr_url.replace('\'', "'\"'\"'")
            );
            std::fs::write(&binary, script).expect("fake gh create or update");
            let mut perms = std::fs::metadata(&binary).expect("metadata").permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                perms.set_mode(0o755);
                std::fs::set_permissions(&binary, perms).expect("chmod");
            }
            FakeGh { binary, argv_path }
        }

        fn fake_provider_repair(&self, stdout: &str, exit_code: i32) -> FakeProviderRepair {
            let binary = self.root.path().join(format!(
                "fake-provider-repair-{}.sh",
                self.root
                    .path()
                    .read_dir()
                    .map(|entries| entries.count())
                    .unwrap_or(0)
            ));
            let context_path_capture = self.root.path().join(format!(
                "fake-provider-context-{}.txt",
                self.root
                    .path()
                    .read_dir()
                    .map(|entries| entries.count())
                    .unwrap_or(0)
            ));
            let script = format!(
                "#!/usr/bin/env bash\nprintf '%s\\n' \"$TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT\" > {}\ncat <<'TAU_PROVIDER_REPAIR_JSON'\n{}\nTAU_PROVIDER_REPAIR_JSON\nexit {}\n",
                shell_single_quote(context_path_capture.display().to_string().as_str()),
                stdout,
                exit_code
            );
            std::fs::write(&binary, script).expect("fake provider");
            let mut perms = std::fs::metadata(&binary).expect("metadata").permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                perms.set_mode(0o755);
                std::fs::set_permissions(&binary, perms).expect("chmod");
            }
            FakeProviderRepair {
                binary,
                context_path_capture,
            }
        }
    }

    struct FakeGh {
        binary: PathBuf,
        argv_path: PathBuf,
    }

    struct FakeProviderRepair {
        binary: PathBuf,
        context_path_capture: PathBuf,
    }

    impl FakeProviderRepair {
        fn policy(
            &self,
            max_attempts: u32,
            provider: &str,
            model: &str,
        ) -> AutonomousCodingProviderRepairPolicy {
            AutonomousCodingProviderRepairPolicy {
                enabled: true,
                max_attempts,
                command: Some(self.binary.clone()),
                args: Vec::new(),
                provider: Some(provider.to_string()),
                model: Some(model.to_string()),
            }
        }
    }

    async fn make_pr_ready_job_with_pr_url(
        fixture: &CodingJobFixture,
        runtime: &AutonomousCodingJobRuntime,
    ) -> AutonomousCodingJobRunOutcome {
        let submitted = runtime
            .submit_job(fixture.submit_request(false, fixture.controlled_edits()))
            .await
            .expect("submit job");
        let outcome = runtime
            .run_or_replay_job(submitted.record.job_id.as_str(), 2_000)
            .expect("run job");
        let mut state = load_coding_mission_state(
            runtime.config().state_dir.as_path(),
            outcome.record.mission_id.as_str(),
        )
        .expect("mission state");
        state
            .pr_ready_bundle
            .as_mut()
            .expect("pr-ready bundle")
            .pr_url = Some("https://github.com/njfio/Tau/pull/3796".to_string());
        save_coding_mission_state(&state).expect("save pr url");
        outcome
    }

    fn mark_background_job_stale_running(state_dir: &Path, background_job_id: &str) {
        let path = state_dir
            .join("jobs")
            .join(format!("{background_job_id}.json"));
        for _ in 0..50 {
            if path.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let raw = std::fs::read_to_string(&path).expect("background manifest");
        let mut value: Value = serde_json::from_str(&raw).expect("manifest json");
        value["status"] = Value::String("running".to_string());
        value["reason_code"] = Value::String("job_started".to_string());
        value["started_unix_ms"] = Value::from(1_u64);
        value["updated_unix_ms"] = Value::from(1_u64);
        value["finished_unix_ms"] = Value::Null;
        value["exit_code"] = Value::Null;
        value["error"] = Value::Null;
        std::fs::write(
            path,
            serde_json::to_string_pretty(&value).expect("serialized manifest"),
        )
        .expect("write stale manifest");
    }

    fn git(repo: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo)
            .output()
            .expect("git command");
        assert!(
            output.status.success(),
            "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn shell_single_quote(value: &str) -> String {
        value.replace('\'', "'\"'\"'")
    }
}
