//! Durable autonomous coding job runtime.
//!
//! This layer links `CodingMissionRunner` with the generic background jobs
//! runtime so coding work can be submitted, resumed, recovered, and inspected
//! as one product loop.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
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
    BackgroundJobCreateRequest, BackgroundJobRecord, BackgroundJobRecoveryReport,
    BackgroundJobRuntime, BackgroundJobRuntimeConfig, BackgroundJobTraceContext,
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
const AUTONOMOUS_CODING_JOB_MAX_REPLAY_PASSES: usize = 6;

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
    pub commit_message: String,
    pub created_unix_ms: u64,
    pub updated_unix_ms: u64,
    pub recovery_count: u64,
    pub replay_count: u64,
    #[serde(default)]
    pub last_background_reason_code: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
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
    pub pr_state: String,
    #[serde(default)]
    pub pr_ready_command: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    pub recovery_count: u64,
    pub replay_count: u64,
    #[serde(default)]
    pub last_background_reason_code: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
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
            commit_message: request.commit_message,
            created_unix_ms: request.started_unix_ms,
            updated_unix_ms: request.started_unix_ms,
            recovery_count: 0,
            replay_count: 0,
            last_background_reason_code: None,
            last_error: None,
        };

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
        self.execute_job(job_id, None, None, started_unix_ms, ReplayCounterMode::Auto)
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
        )
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

    fn execute_job(
        &self,
        job_id: &str,
        controlled_edits_override: Option<Vec<CodingMissionControlledEdit>>,
        commit_message_override: Option<String>,
        started_unix_ms: u64,
        replay_counter_mode: ReplayCounterMode,
    ) -> Result<AutonomousCodingJobRunOutcome> {
        let mut record = load_autonomous_coding_job_record(&self.config.state_dir, job_id)?;
        let controlled_edits =
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
        record.last_error = None;
        persist_autonomous_coding_job_record(&record)?;
        self.refresh_status_snapshot(&record)?;

        let runner = CodingMissionRunner::new();
        let mut pr_ready_bundle = None;
        let mut last_phase = state_before.phase;
        let mut final_state = state_before;
        for pass in 0..AUTONOMOUS_CODING_JOB_MAX_REPLAY_PASSES {
            let pass_started = started_unix_ms.saturating_add((pass as u64).saturating_mul(1_000));
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
                )?);
                record.status = AutonomousCodingJobStatus::PrReady;
                record.reason_code = AUTONOMOUS_CODING_JOB_REASON_PR_READY.to_string();
                record.updated_unix_ms = pass_started.saturating_add(999);
                record.last_error = None;
                persist_autonomous_coding_job_record(&record)?;
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
                record.last_error = Some(reason);
                persist_autonomous_coding_job_record(&record)?;
                let status = self.refresh_status_snapshot(&record)?;
                return Ok(AutonomousCodingJobRunOutcome {
                    record,
                    status,
                    pr_ready_bundle,
                });
            }

            let waiting_for_edit =
                final_state
                    .resume_checkpoint
                    .as_ref()
                    .is_some_and(|checkpoint| {
                        checkpoint.next_action
                            == tau_agent_core::CodingMissionResumeAction::ApplyEdit
                            && controlled_edits.is_empty()
                    });
            if waiting_for_edit {
                record.status = AutonomousCodingJobStatus::Running;
                record.reason_code = AUTONOMOUS_CODING_JOB_REASON_WAITING_FOR_EDIT.to_string();
                record.updated_unix_ms = pass_started.saturating_add(999);
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
        record.last_error = Some("replay_pass_limit_exhausted".to_string());
        persist_autonomous_coding_job_record(&record)?;
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

fn ensure_autonomous_coding_job_layout(state_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(state_dir.join("autonomous-coding-jobs"))?;
    std::fs::create_dir_all(state_dir.join("coding-missions"))?;
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
        pr_state,
        pr_ready_command: pr_ready_bundle.map(|bundle| bundle.manual_gh_pr_create_command.clone()),
        pr_url: pr_ready_bundle.and_then(|bundle| bundle.pr_url.clone()),
        recovery_count: record.recovery_count,
        replay_count: record.replay_count,
        last_background_reason_code: record.last_background_reason_code.clone(),
        last_error: record.last_error.clone(),
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
        ]),
    }
}

fn ensure_pr_ready_bundle(
    state: &mut CodingMissionState,
    record: &AutonomousCodingJobRecord,
    started_unix_ms: u64,
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
            github_env: BTreeMap::new(),
            gh_binary: None,
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
}
