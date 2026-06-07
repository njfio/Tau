//! Standalone binary: `tau-live-coding-loop-harness`.
//!
//! Drives the live `CodingMissionRunner` over a disposable repository for the
//! M334 repo-spec-to-PR benchmark task.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use tau_agent_core::{
    load_autonomy_benchmark_fixture, CodingGitLifecycleEvidenceKind, CodingMissionConfig,
    CodingMissionControlledEdit, CodingMissionPhase, CodingMissionPrMode,
    CodingMissionPrPublicationStatus, CodingMissionPrReadyBundle, CodingMissionPrReadyRequest,
    CodingMissionResumeAction, CodingMissionResumeRequest, CodingMissionRunOutcome,
    CodingMissionRunRequest, CodingMissionRunner, CodingMissionState,
    CodingWorkspaceCommandEvidence, CodingWorkspaceCommandStatus,
};

#[derive(Debug, Parser)]
#[command(
    name = "tau-live-coding-loop-harness",
    about = "Run the live CodingMissionRunner against the M334 repo-spec-to-PR benchmark task"
)]
struct Args {
    /// Benchmark fixture JSON. Defaults to the repo's canonical M334 fixture.
    #[arg(long)]
    fixture: Option<PathBuf>,

    /// Benchmark task id to bind to the coding mission.
    #[arg(long, default_value = "repo_spec_to_pr_feature_delivery")]
    task_id: String,

    /// Harness mode to run.
    #[arg(long, value_enum)]
    mode: HarnessMode,

    /// Root for CodingMissionState persistence.
    #[arg(long)]
    state_root: PathBuf,

    /// Disposable git repository root.
    #[arg(long)]
    repo_root: PathBuf,

    /// Output file for the JSON report.
    #[arg(long)]
    output: PathBuf,

    /// Stable run id for deterministic report fields.
    #[arg(long, default_value = "m334-live-coding-loop")]
    run_id: String,

    /// Override timestamps for reproducible tests.
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum HarnessMode {
    Success,
    Resume,
    Blocked,
}

impl HarnessMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Resume => "resume",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Serialize)]
struct LiveLoopReport {
    schema_version: u32,
    benchmark_id: String,
    task_id: String,
    task_goal: String,
    mode: String,
    passed: bool,
    failure_reasons: Vec<String>,
    mission_id: String,
    session_key: String,
    phase: &'static str,
    repo_path: String,
    state_root: String,
    branch: String,
    commit_hash: Option<String>,
    changed_files: Vec<String>,
    verifier_transcript: Vec<VerifierEvidenceReport>,
    resume: Option<ResumeReport>,
    blocked_reason: Option<String>,
    pr_ready: Option<PrReadyReport>,
    operator_interventions_used: Vec<String>,
    no_routine_human_steering_used: bool,
}

#[derive(Debug, Serialize)]
struct VerifierEvidenceReport {
    command_id: String,
    status: &'static str,
    reason_code: String,
    exit_status: Option<i32>,
    argv: Vec<String>,
    stdout_path: String,
    stderr_path: String,
}

#[derive(Debug, Serialize)]
struct ResumeReport {
    crash_injected: bool,
    first_phase: &'static str,
    checkpoint_action: Option<&'static str>,
    resume_action: &'static str,
    restored_branch: Option<String>,
}

#[derive(Debug, Serialize)]
struct PrReadyReport {
    status: &'static str,
    branch_name: String,
    commit_hash: Option<String>,
    body_path: String,
    manual_gh_pr_create_command: String,
    pr_url: Option<String>,
}

fn main() -> ExitCode {
    let mut args = Args::parse();
    let fixture_path = args.fixture.take().unwrap_or_else(default_fixture_path);
    let started_unix_ms = args.started_unix_ms.unwrap_or_else(now_unix_ms);

    match run(args, fixture_path, started_unix_ms) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(error) => {
            eprintln!("tau-live-coding-loop-harness: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Args, fixture_path: PathBuf, started_unix_ms: u64) -> anyhow::Result<bool> {
    let fixture = load_autonomy_benchmark_fixture(&fixture_path)
        .with_context(|| format!("load benchmark fixture {}", fixture_path.display()))?;
    let task = fixture
        .tasks
        .iter()
        .find(|task| task.id == args.task_id)
        .with_context(|| format!("benchmark task '{}' not found", args.task_id))?;
    fs::create_dir_all(&args.state_root)
        .with_context(|| format!("create state root {}", args.state_root.display()))?;
    prepare_disposable_repo(&args.repo_root)?;

    let mission_id = format!("{}-{}", args.run_id, args.mode.as_str());
    let verifier_command = match args.mode {
        HarnessMode::Success | HarnessMode::Resume => "grep -q pass status.txt",
        HarnessMode::Blocked => "definitely-not-a-tau-command",
    };
    let config = CodingMissionConfig {
        state_root: args.state_root.clone(),
        mission_id: mission_id.clone(),
        session_key: format!("session-{mission_id}"),
        repo_path: args.repo_root.clone(),
        issue_url: Some("https://github.com/njfio/Tau/issues/3654".to_string()),
        goal: task.goal.clone(),
        base_branch: "master".to_string(),
        branch_prefix: "codex/issue-3654-live-loop".to_string(),
        verifier_commands: vec![verifier_command.to_string()],
        pr_mode: CodingMissionPrMode::PrReady,
        allowed_roots: vec![args.repo_root.clone(), args.state_root.clone()],
        created_unix_ms: started_unix_ms,
    };
    let mut state = CodingMissionState::create(config)?;
    let runner = CodingMissionRunner::new();

    let mut resume_report = None;
    let outcome = match args.mode {
        HarnessMode::Success => run_success_case(&runner, &mut state, started_unix_ms)?,
        HarnessMode::Resume => {
            let (outcome, resume) = run_resume_case(&runner, &mut state, started_unix_ms)?;
            resume_report = Some(resume);
            outcome
        }
        HarnessMode::Blocked => run_blocked_case(&runner, &mut state, started_unix_ms)?,
    };
    let pr_ready = if state.phase == CodingMissionPhase::PrReady {
        Some(
            state
                .prepare_pr_ready_bundle(CodingMissionPrReadyRequest {
                    title: Some("M334 live coding loop fixture".to_string()),
                    risk_notes: vec!["Risk: disposable local fixture only".to_string()],
                    rollback_notes: vec!["Rollback: delete the disposable repo".to_string()],
                    allow_draft_pr: false,
                    github_env: BTreeMap::new(),
                    gh_binary: None,
                    started_unix_ms: started_unix_ms + 9_000,
                })
                .context("prepare pr-ready bundle")?,
        )
    } else {
        None
    };
    let report = build_report(
        &fixture.benchmark_id,
        &task.id,
        args.mode,
        &state,
        &outcome,
        resume_report,
        pr_ready.as_ref(),
    );
    let passed = report.passed;
    write_report(&args.output, &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(passed)
}

fn run_success_case(
    runner: &CodingMissionRunner,
    state: &mut CodingMissionState,
    started_unix_ms: u64,
) -> anyhow::Result<CodingMissionRunOutcome> {
    runner
        .run(
            state,
            CodingMissionRunRequest {
                controlled_edit: Some(pass_status_edit("controlled_fix_for_live_loop")),
                commit_message: "Make M334 live verifier green".to_string(),
                started_unix_ms: started_unix_ms + 1_000,
            },
        )
        .context("run success coding mission")
}

fn run_resume_case(
    runner: &CodingMissionRunner,
    state: &mut CodingMissionState,
    started_unix_ms: u64,
) -> anyhow::Result<(CodingMissionRunOutcome, ResumeReport)> {
    let first = runner
        .run(
            state,
            CodingMissionRunRequest {
                controlled_edit: None,
                commit_message: "Make M334 live verifier green".to_string(),
                started_unix_ms: started_unix_ms + 2_000,
            },
        )
        .context("run red checkpoint before resume")?;
    let checkpoint_action = state
        .resume_checkpoint
        .as_ref()
        .map(|checkpoint| resume_action_label(checkpoint.next_action));
    run_git(&state.repo_path, &["switch", "master"]).context("simulate crash on master")?;
    let resumed = runner
        .resume(CodingMissionResumeRequest {
            state_root: state.state_root.clone(),
            mission_id: state.mission_id.clone(),
            controlled_edit: Some(pass_status_edit("controlled_fix_after_live_resume")),
            commit_message: "Make M334 live verifier green after resume".to_string(),
            started_unix_ms: started_unix_ms + 3_000,
            stop_after: None,
        })
        .context("resume coding mission")?;
    *state = resumed.state;
    let report = ResumeReport {
        crash_injected: true,
        first_phase: phase_label(first.phase),
        checkpoint_action,
        resume_action: resume_action_label(resumed.resume_action),
        restored_branch: resumed.restored_branch,
    };
    Ok((resumed.run, report))
}

fn run_blocked_case(
    runner: &CodingMissionRunner,
    state: &mut CodingMissionState,
    started_unix_ms: u64,
) -> anyhow::Result<CodingMissionRunOutcome> {
    runner
        .run(
            state,
            CodingMissionRunRequest {
                controlled_edit: None,
                commit_message: "Blocked verifier should not commit".to_string(),
                started_unix_ms: started_unix_ms + 4_000,
            },
        )
        .context("run blocked coding mission")
}

fn pass_status_edit(reason_code: &str) -> CodingMissionControlledEdit {
    CodingMissionControlledEdit {
        relative_path: PathBuf::from("status.txt"),
        contents: "pass\n".to_string(),
        reason_code: reason_code.to_string(),
    }
}

fn build_report(
    benchmark_id: &str,
    task_id: &str,
    mode: HarnessMode,
    state: &CodingMissionState,
    outcome: &CodingMissionRunOutcome,
    resume: Option<ResumeReport>,
    pr_ready: Option<&CodingMissionPrReadyBundle>,
) -> LiveLoopReport {
    let commit = state
        .git_evidence
        .iter()
        .rev()
        .find(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated);
    let changed_files = pr_ready
        .map(|bundle| bundle.changed_files.clone())
        .or_else(|| commit.map(|evidence| evidence.changed_files.clone()))
        .unwrap_or_default();
    let branch = state
        .resume_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.branch_name.clone())
        .or_else(|| pr_ready.map(|bundle| bundle.branch_name.clone()))
        .or_else(|| commit.map(|evidence| evidence.branch_name.clone()))
        .unwrap_or_else(|| current_branch(&state.repo_path).unwrap_or_default());
    let verifier_transcript = state
        .command_evidence
        .iter()
        .filter(|evidence| evidence.reason_code.starts_with("coding_verifier"))
        .map(verifier_report)
        .collect::<Vec<_>>();
    let pr_ready_report = pr_ready.map(|bundle| PrReadyReport {
        status: pr_status_label(bundle.status),
        branch_name: bundle.branch_name.clone(),
        commit_hash: bundle.commit_hash.clone(),
        body_path: bundle.body_path.display().to_string(),
        manual_gh_pr_create_command: bundle.manual_gh_pr_create_command.clone(),
        pr_url: bundle.pr_url.clone(),
    });
    let mut failure_reasons = Vec::new();
    let passed = match mode {
        HarnessMode::Success | HarnessMode::Resume => {
            let ok = state.phase == CodingMissionPhase::PrReady
                && pr_ready_report.is_some()
                && commit
                    .and_then(|evidence| evidence.commit_hash.as_ref())
                    .is_some()
                && verifier_transcript
                    .iter()
                    .any(|evidence| evidence.status == "failed")
                && verifier_transcript
                    .iter()
                    .any(|evidence| evidence.status == "succeeded");
            if !ok {
                failure_reasons
                    .push("success_or_resume_case_missing_required_evidence".to_string());
            }
            ok
        }
        HarnessMode::Blocked => {
            let ok = state.phase == CodingMissionPhase::Blocked
                && outcome.blocked_reason.as_deref() == Some("verifier_command_failed_to_start")
                && commit.is_none()
                && pr_ready_report.is_none();
            if !ok {
                failure_reasons.push("blocked_case_did_not_fail_closed".to_string());
            }
            ok
        }
    };

    LiveLoopReport {
        schema_version: 1,
        benchmark_id: benchmark_id.to_string(),
        task_id: task_id.to_string(),
        task_goal: state.goal.clone(),
        mode: mode.as_str().to_string(),
        passed,
        failure_reasons,
        mission_id: state.mission_id.clone(),
        session_key: state.session_key.clone(),
        phase: phase_label(state.phase),
        repo_path: state.repo_path.display().to_string(),
        state_root: state.state_root.display().to_string(),
        branch,
        commit_hash: commit.and_then(|evidence| evidence.commit_hash.clone()),
        changed_files,
        verifier_transcript,
        resume,
        blocked_reason: outcome.blocked_reason.clone(),
        pr_ready: pr_ready_report,
        operator_interventions_used: Vec::new(),
        no_routine_human_steering_used: true,
    }
}

fn verifier_report(evidence: &CodingWorkspaceCommandEvidence) -> VerifierEvidenceReport {
    VerifierEvidenceReport {
        command_id: evidence.command_id.clone(),
        status: command_status_label(evidence.status),
        reason_code: evidence.reason_code.clone(),
        exit_status: evidence.exit_status,
        argv: evidence.argv.clone(),
        stdout_path: evidence.stdout_path.display().to_string(),
        stderr_path: evidence.stderr_path.display().to_string(),
    }
}

fn prepare_disposable_repo(repo_root: &Path) -> anyhow::Result<()> {
    if repo_root.exists() {
        bail!("repo root already exists: {}", repo_root.display());
    }
    fs::create_dir_all(repo_root)
        .with_context(|| format!("create repo root {}", repo_root.display()))?;
    let repo_arg = repo_root.to_string_lossy().to_string();
    run_command(None, "git", &["init", repo_arg.as_str()]).context("git init")?;
    run_git(repo_root, &["checkout", "-B", "master"]).context("create master branch")?;
    run_git(repo_root, &["config", "user.email", "tau@example.invalid"])?;
    run_git(repo_root, &["config", "user.name", "Tau Live Harness"])?;
    fs::write(repo_root.join("README.md"), "# Tau live loop fixture\n")?;
    fs::write(repo_root.join("status.txt"), "fail\n")?;
    run_git(repo_root, &["add", "README.md", "status.txt"])?;
    run_git(repo_root, &["commit", "-m", "seed live coding fixture"])?;
    Ok(())
}

fn run_git(repo_root: &Path, args: &[&str]) -> anyhow::Result<String> {
    run_command(Some(repo_root), "git", args)
}

fn run_command(cwd: Option<&Path>, program: &str, args: &[&str]) -> anyhow::Result<String> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to start command: {program} {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "command failed: {program} {}\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn current_branch(repo_root: &Path) -> anyhow::Result<String> {
    run_git(repo_root, &["branch", "--show-current"])
}

fn write_report(path: &Path, report: &LiveLoopReport) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create report parent {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(report)? + "\n")
        .with_context(|| format!("write report {}", path.display()))
}

fn command_status_label(status: CodingWorkspaceCommandStatus) -> &'static str {
    match status {
        CodingWorkspaceCommandStatus::Succeeded => "succeeded",
        CodingWorkspaceCommandStatus::Failed => "failed",
        CodingWorkspaceCommandStatus::Denied => "denied",
    }
}

fn phase_label(phase: CodingMissionPhase) -> &'static str {
    match phase {
        CodingMissionPhase::Intake => "intake",
        CodingMissionPhase::Planned => "planned",
        CodingMissionPhase::PreparingBranch => "preparing_branch",
        CodingMissionPhase::Executing => "executing",
        CodingMissionPhase::Verifying => "verifying",
        CodingMissionPhase::PrReady => "pr_ready",
        CodingMissionPhase::Blocked => "blocked",
        CodingMissionPhase::Completed => "completed",
    }
}

fn resume_action_label(action: CodingMissionResumeAction) -> &'static str {
    match action {
        CodingMissionResumeAction::RunVerifier => "run_verifier",
        CodingMissionResumeAction::ApplyEdit => "apply_edit",
        CodingMissionResumeAction::VerifyAfterEdit => "verify_after_edit",
        CodingMissionResumeAction::Commit => "commit",
        CodingMissionResumeAction::Blocked => "blocked",
        CodingMissionResumeAction::PrReady => "pr_ready",
    }
}

fn pr_status_label(status: CodingMissionPrPublicationStatus) -> &'static str {
    match status {
        CodingMissionPrPublicationStatus::ManualReady => "manual_ready",
        CodingMissionPrPublicationStatus::DraftCreated => "draft_created",
        CodingMissionPrPublicationStatus::DraftFailed => "draft_failed",
    }
}

fn default_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tasks/fixtures/m334/tranche-one-autonomy-benchmark.json")
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}
