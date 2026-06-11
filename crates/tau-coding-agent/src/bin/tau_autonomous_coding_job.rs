//! Standalone autonomous coding job operator binary.
//!
//! This is the narrow product surface for submitting, running, replaying, and
//! inspecting durable coding jobs while the main control plane wiring catches
//! up.

use std::{
    path::PathBuf,
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;
use tau_agent_core::{CodingMissionControlledEdit, CodingMissionPrMode};
use tau_runtime::{
    AutonomousCodingAutoMergeRequest, AutonomousCodingIssueIntakeDecision,
    AutonomousCodingIssueIntakeOutcome, AutonomousCodingIssueIntakeRequest,
    AutonomousCodingIssueToMergeRequest, AutonomousCodingJobMarkBlockedRequest,
    AutonomousCodingJobReplayRequest, AutonomousCodingJobRuntime, AutonomousCodingJobRuntimeConfig,
    AutonomousCodingJobSubmitRequest, AutonomousCodingMergeMethod,
    AutonomousCodingProviderRepairPolicy,
};

#[path = "tau_autonomous_coding_job/openrouter_repair_adapter.rs"]
mod openrouter_repair_adapter;

#[derive(Debug, Parser)]
#[command(
    name = "tau-autonomous-coding-job",
    about = "Submit, run, replay, recover, and inspect durable Tau coding jobs"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Persist a coding mission job and optionally enqueue its background worker.
    Submit(Box<JobSubmitArgs>),
    /// Run or resume a persisted autonomous coding job.
    Run(Box<JobRunArgs>),
    /// Explicitly replay a checkpoint with optional updated edits/message.
    Replay(Box<JobReplayArgs>),
    /// Emit the operator status JSON for a persisted job.
    Status(Box<JobStatusArgs>),
    /// Run the background-job stuck recovery sweep and refresh linked jobs.
    Recover(Box<JobRecoverArgs>),
    /// Safely mark a job blocked with operator evidence.
    MarkBlocked(Box<JobMarkBlockedArgs>),
    /// Request GitHub auto-merge for a PR-ready autonomous coding job.
    AutoMerge(Box<JobAutoMergeArgs>),
    /// Ingest an arbitrary issue without verifier/edit authority.
    IntakeIssue(Box<IssueIntakeArgs>),
    /// Emit a persisted issue-intake authority plan.
    IntakeStatus(Box<IssueIntakeStatusArgs>),
    /// Run issue-to-merge from a persisted intake after authority is supplied.
    IntakeRun(Box<IssueIntakeRunArgs>),
    /// Run issue intake through PR-ready output and optional guarded auto-merge.
    IssueToMerge(Box<IssueToMergeArgs>),
    /// Built-in OpenRouter repair adapter for durable coding jobs.
    OpenrouterRepairAdapter(Box<OpenrouterRepairAdapterArgs>),
}

#[derive(Debug, Parser)]
struct JobSubmitArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[command(flatten)]
    provider_repair: ProviderRepairArgs,

    #[arg(long)]
    repo_path: PathBuf,
    #[arg(long)]
    mission_id: String,
    #[arg(long, default_value = "local-session")]
    session_key: String,
    #[arg(long)]
    issue_url: Option<String>,
    #[arg(long)]
    goal: String,
    #[arg(long, default_value = "master")]
    base_branch: String,
    #[arg(long, default_value = "codex/autonomous-coding-job-")]
    branch_prefix: String,
    #[arg(long = "verifier-command", required = true)]
    verifier_commands: Vec<String>,
    #[arg(long = "allowed-root")]
    allowed_roots: Vec<PathBuf>,
    #[arg(long = "edit")]
    edits: Vec<String>,
    #[arg(long)]
    commit_message: String,
    #[arg(long, value_enum, default_value = "draft")]
    pr_mode: PrModeArg,
    #[arg(long)]
    enqueue_background_job: bool,
    #[arg(long)]
    timeout_ms: Option<u64>,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct JobRunArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    job_id: String,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct JobReplayArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    job_id: String,
    #[arg(long = "edit")]
    edits: Vec<String>,
    #[arg(long)]
    commit_message: Option<String>,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct JobStatusArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    job_id: String,
}

#[derive(Debug, Parser)]
struct JobRecoverArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
}

#[derive(Debug, Parser)]
struct JobMarkBlockedArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    job_id: String,
    #[arg(long, default_value = "operator_marked_blocked")]
    reason_code: String,
    #[arg(long, default_value = "operator marked job blocked")]
    detail: String,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct JobAutoMergeArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    job_id: String,
    #[arg(long)]
    allow_auto_merge: bool,
    #[arg(long, value_enum, default_value = "squash")]
    merge_method: MergeMethodArg,
    #[arg(long)]
    delete_branch: bool,
    #[arg(long)]
    gh_binary: Option<PathBuf>,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct IssueIntakeArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    intake_id: String,
    #[arg(long)]
    issue_url: String,
    #[arg(long)]
    issue_title: String,
    #[arg(long)]
    issue_body: String,
    #[arg(long)]
    repo_path: PathBuf,
    #[arg(long, default_value = "master")]
    base_branch: String,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct IssueIntakeStatusArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[arg(long)]
    intake_id: String,
}

#[derive(Debug, Parser)]
struct IssueIntakeRunArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[command(flatten)]
    provider_repair: ProviderRepairArgs,

    #[arg(long)]
    intake_id: String,
    #[arg(long)]
    mission_id: Option<String>,
    #[arg(long, default_value = "local-session")]
    session_key: String,
    #[arg(long)]
    goal: Option<String>,
    #[arg(long, default_value = "codex/autonomous-coding-job-")]
    branch_prefix: String,
    #[arg(long = "verifier-command")]
    verifier_commands: Vec<String>,
    #[arg(long = "allowed-root")]
    allowed_roots: Vec<PathBuf>,
    #[arg(long = "edit")]
    edits: Vec<String>,
    #[arg(long)]
    commit_message: Option<String>,
    #[arg(long, value_enum, default_value = "draft")]
    pr_mode: PrModeArg,
    #[arg(long)]
    timeout_ms: Option<u64>,
    #[arg(long)]
    allow_auto_merge: bool,
    #[arg(long, value_enum, default_value = "squash")]
    merge_method: MergeMethodArg,
    #[arg(long)]
    delete_branch: bool,
    #[arg(long)]
    gh_binary: Option<PathBuf>,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Parser)]
struct IssueToMergeArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,
    #[command(flatten)]
    provider_repair: ProviderRepairArgs,

    #[arg(long)]
    intake_id: String,
    #[arg(long)]
    repo_path: PathBuf,
    #[arg(long)]
    mission_id: String,
    #[arg(long, default_value = "local-session")]
    session_key: String,
    #[arg(long)]
    issue_url: String,
    #[arg(long)]
    issue_title: String,
    #[arg(long)]
    issue_body: String,
    #[arg(long)]
    goal: Option<String>,
    #[arg(long, default_value = "master")]
    base_branch: String,
    #[arg(long, default_value = "codex/autonomous-coding-job-")]
    branch_prefix: String,
    #[arg(long = "verifier-command")]
    verifier_commands: Vec<String>,
    #[arg(long = "allowed-root")]
    allowed_roots: Vec<PathBuf>,
    #[arg(long = "edit")]
    edits: Vec<String>,
    #[arg(long)]
    commit_message: Option<String>,
    #[arg(long, value_enum, default_value = "draft")]
    pr_mode: PrModeArg,
    #[arg(long)]
    timeout_ms: Option<u64>,
    #[arg(long)]
    allow_auto_merge: bool,
    #[arg(long, value_enum, default_value = "squash")]
    merge_method: MergeMethodArg,
    #[arg(long)]
    delete_branch: bool,
    #[arg(long)]
    gh_binary: Option<PathBuf>,
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Parser)]
struct RuntimeArgs {
    #[arg(long, default_value = ".tau/autonomous-coding")]
    state_dir: PathBuf,
    #[arg(long, default_value = ".tau/jobs")]
    jobs_state_dir: PathBuf,
    #[arg(long)]
    runner_command: Option<PathBuf>,
}

#[derive(Debug, Clone, Parser)]
struct ProviderRepairArgs {
    #[arg(long)]
    provider_repair_command: Option<PathBuf>,
    #[arg(long = "provider-repair-arg")]
    provider_repair_args: Vec<String>,
    #[arg(long, default_value_t = 0)]
    provider_repair_attempts: u32,
    #[arg(long)]
    provider_repair_provider: Option<String>,
    #[arg(long)]
    provider_repair_model: Option<String>,
    #[arg(long)]
    provider_repair_openrouter: bool,
    #[arg(long)]
    provider_repair_env_file: Option<PathBuf>,
    #[arg(long)]
    provider_repair_api_base: Option<String>,
    #[arg(long, default_value_t = 120_000)]
    provider_repair_timeout_ms: u64,
    #[arg(long, default_value_t = 1_200)]
    provider_repair_max_tokens: u32,
    #[arg(long, default_value_t = 1)]
    provider_repair_max_retries: usize,
}

#[derive(Debug, Parser)]
struct OpenrouterRepairAdapterArgs {
    #[arg(long)]
    context: Option<PathBuf>,
    #[arg(long)]
    env_file: Option<PathBuf>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    api_base: Option<String>,
    #[arg(long, default_value_t = 120_000)]
    request_timeout_ms: u64,
    #[arg(long, default_value_t = 1_200)]
    max_tokens: u32,
    #[arg(long, default_value_t = 1)]
    max_retries: usize,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PrModeArg {
    Disabled,
    #[value(name = "pr-ready")]
    PrReady,
    Draft,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum MergeMethodArg {
    Merge,
    Squash,
    Rebase,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Args::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tau-autonomous-coding-job: {error:#}");
            ExitCode::from(2)
        }
    }
}

async fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Submit(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let allowed_roots = if args.allowed_roots.is_empty() {
                vec![args.repo_path.clone()]
            } else {
                args.allowed_roots
            };
            let outcome = runtime
                .submit_job(AutonomousCodingJobSubmitRequest {
                    mission_id: args.mission_id,
                    session_key: args.session_key,
                    repo_path: args.repo_path,
                    issue_url: args.issue_url,
                    goal: args.goal,
                    base_branch: args.base_branch,
                    branch_prefix: args.branch_prefix,
                    verifier_commands: args.verifier_commands,
                    pr_mode: args.pr_mode.into(),
                    allowed_roots,
                    controlled_edits: parse_edits(&args.edits)?,
                    provider_repair: provider_repair_policy(&args.provider_repair)?,
                    commit_message: args.commit_message,
                    enqueue_background_job: args.enqueue_background_job,
                    timeout_ms: args.timeout_ms,
                    started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
                })
                .await?;
            print_json(&outcome)?;
        }
        Command::Run(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = runtime.run_or_replay_job(
                args.job_id.as_str(),
                args.started_unix_ms.unwrap_or_else(now_unix_ms),
            )?;
            print_json(&outcome)?;
        }
        Command::Replay(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = runtime.replay_job(AutonomousCodingJobReplayRequest {
                job_id: args.job_id,
                controlled_edits: parse_edits(&args.edits)?,
                commit_message: args.commit_message,
                started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
            })?;
            print_json(&outcome)?;
        }
        Command::Status(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let status = runtime.status(args.job_id.as_str())?;
            print_json(&status)?;
        }
        Command::Recover(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = runtime.recover_stuck_background_jobs().await?;
            print_json(&json!({
                "background_report": {
                    "scanned_running": outcome.background_report.scanned_running,
                    "recovered": outcome.background_report.recovered,
                    "skipped_fresh": outcome.background_report.skipped_fresh,
                    "skipped_cancelled": outcome.background_report.skipped_cancelled,
                    "recovered_job_ids": outcome.background_report.recovered_job_ids,
                },
                "recovered_jobs": outcome.recovered_jobs,
            }))?;
        }
        Command::MarkBlocked(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = runtime.mark_job_blocked(AutonomousCodingJobMarkBlockedRequest {
                job_id: args.job_id,
                reason_code: args.reason_code,
                detail: args.detail,
                started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
            })?;
            print_json(&outcome)?;
        }
        Command::AutoMerge(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = runtime.request_auto_merge(AutonomousCodingAutoMergeRequest {
                job_id: args.job_id,
                allow_auto_merge: args.allow_auto_merge,
                merge_method: args.merge_method.into(),
                delete_branch: args.delete_branch,
                github_env: github_env_from_process(),
                gh_binary: args.gh_binary,
                started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
            })?;
            print_json(&outcome)?;
        }
        Command::IntakeIssue(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome =
                runtime.intake_issue_without_authority(AutonomousCodingIssueIntakeRequest {
                    intake_id: args.intake_id,
                    issue_url: args.issue_url,
                    issue_title: args.issue_title,
                    issue_body: args.issue_body,
                    repo_path: args.repo_path,
                    base_branch: args.base_branch,
                    started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
                })?;
            print_json(&outcome)?;
        }
        Command::IntakeStatus(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = runtime.issue_intake_status(args.intake_id.as_str())?;
            print_json(&outcome)?;
        }
        Command::IntakeRun(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let outcome = run_intake_run(&runtime, args).await?;
            print_json(&outcome)?;
        }
        Command::IssueToMerge(args) => {
            let args = *args;
            let runtime = runtime_from_args(&args.runtime)?;
            let goal = args.goal.unwrap_or_else(|| {
                format!("{}\n\n{}", args.issue_title.trim(), args.issue_body.trim())
            });
            let commit_message = args
                .commit_message
                .unwrap_or_else(|| format!("Resolve {}", args.issue_title.trim()));
            let allowed_roots = if args.allowed_roots.is_empty() {
                vec![args.repo_path.clone()]
            } else {
                args.allowed_roots
            };
            let outcome = runtime
                .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
                    intake_id: args.intake_id,
                    mission_id: args.mission_id,
                    session_key: args.session_key,
                    repo_path: args.repo_path,
                    issue_url: args.issue_url,
                    issue_title: args.issue_title,
                    issue_body: args.issue_body,
                    goal,
                    base_branch: args.base_branch,
                    branch_prefix: args.branch_prefix,
                    verifier_commands: args.verifier_commands,
                    pr_mode: args.pr_mode.into(),
                    allowed_roots,
                    controlled_edits: parse_edits(&args.edits)?,
                    provider_repair: provider_repair_policy(&args.provider_repair)?,
                    commit_message,
                    timeout_ms: args.timeout_ms,
                    allow_auto_merge: args.allow_auto_merge,
                    merge_method: args.merge_method.into(),
                    delete_branch: args.delete_branch,
                    github_env: github_env_from_process(),
                    gh_binary: args.gh_binary,
                    started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
                })
                .await?;
            print_json(&outcome)?;
        }
        Command::OpenrouterRepairAdapter(args) => {
            let args = *args;
            openrouter_repair_adapter::run_openrouter_repair_adapter(
                openrouter_repair_adapter::OpenrouterRepairAdapterConfig {
                    context_path: args.context,
                    env_file: args.env_file,
                    model: args.model,
                    api_base: args.api_base,
                    request_timeout_ms: args.request_timeout_ms,
                    max_tokens: args.max_tokens,
                    max_retries: args.max_retries,
                },
            )
            .await?;
        }
    }
    Ok(())
}

fn runtime_from_args(args: &RuntimeArgs) -> Result<AutonomousCodingJobRuntime> {
    let runner_command = match args.runner_command.clone() {
        Some(path) => path,
        None => {
            std::env::current_exe().unwrap_or_else(|_| PathBuf::from("tau_autonomous_coding_job"))
        }
    };
    AutonomousCodingJobRuntime::new(AutonomousCodingJobRuntimeConfig {
        state_dir: args.state_dir.clone(),
        background_jobs_state_dir: args.jobs_state_dir.clone(),
        runner_command,
        ..AutonomousCodingJobRuntimeConfig::default()
    })
}

fn parse_edits(raw_edits: &[String]) -> Result<Vec<CodingMissionControlledEdit>> {
    raw_edits
        .iter()
        .map(|raw| {
            let (path, contents) = raw
                .split_once('=')
                .ok_or_else(|| anyhow!("--edit must use relative/path=contents"))?;
            if path.trim().is_empty() {
                return Err(anyhow!("--edit path must not be empty"));
            }
            Ok(CodingMissionControlledEdit {
                relative_path: PathBuf::from(path),
                contents: contents.to_string(),
                reason_code: "operator_controlled_edit".to_string(),
            })
        })
        .collect()
}

async fn run_intake_run(
    runtime: &AutonomousCodingJobRuntime,
    args: IssueIntakeRunArgs,
) -> Result<tau_runtime::AutonomousCodingIssueToMergeOutcome> {
    let intake = runtime.issue_intake_status(args.intake_id.as_str())?;
    let request = build_intake_run_request(intake, args)?;
    runtime.run_issue_to_merge(request).await
}

fn build_intake_run_request(
    intake: AutonomousCodingIssueIntakeOutcome,
    args: IssueIntakeRunArgs,
) -> Result<AutonomousCodingIssueToMergeRequest> {
    let controlled_edits = parse_edits(&args.edits)?;
    let provider_repair = provider_repair_policy(&args.provider_repair)?;
    if controlled_edits.is_empty() && !provider_repair.is_configured() {
        bail!("intake-run requires --edit or provider repair authority");
    }
    let verifier_commands = intake_run_verifier_commands(&intake, &args.verifier_commands)?;
    let issue_body = intake.issue_body.trim();
    if issue_body.is_empty() {
        bail!("intake-run requires persisted issue_body; rerun intake first");
    }
    let issue_url = intake.issue_url.trim();
    if issue_url.is_empty() {
        bail!("intake-run requires persisted issue_url");
    }
    let issue_title = intake.issue_title.trim();
    if issue_title.is_empty() {
        bail!("intake-run requires persisted issue_title");
    }
    if intake.repo_path.as_os_str().is_empty() {
        bail!("intake-run requires persisted repo_path");
    }
    ensure_intake_run_decision(&intake)?;

    let repo_path = intake.repo_path.clone();
    let allowed_roots = if args.allowed_roots.is_empty() {
        vec![repo_path.clone()]
    } else {
        args.allowed_roots
    };
    let goal = args
        .goal
        .unwrap_or_else(|| format!("{}\n\n{}", issue_title, issue_body));
    let commit_message = args
        .commit_message
        .unwrap_or_else(|| format!("Resolve {}", issue_title));
    let mission_id = args
        .mission_id
        .unwrap_or_else(|| format!("{}-mission", intake.intake_id));

    Ok(AutonomousCodingIssueToMergeRequest {
        intake_id: intake.intake_id,
        mission_id,
        session_key: args.session_key,
        repo_path,
        issue_url: issue_url.to_string(),
        issue_title: issue_title.to_string(),
        issue_body: issue_body.to_string(),
        goal,
        base_branch: intake.base_branch,
        branch_prefix: args.branch_prefix,
        verifier_commands,
        pr_mode: args.pr_mode.into(),
        allowed_roots,
        controlled_edits,
        provider_repair,
        commit_message,
        timeout_ms: args.timeout_ms,
        allow_auto_merge: args.allow_auto_merge,
        merge_method: args.merge_method.into(),
        delete_branch: args.delete_branch,
        github_env: github_env_from_process(),
        gh_binary: args.gh_binary,
        started_unix_ms: args.started_unix_ms.unwrap_or_else(now_unix_ms),
    })
}

fn ensure_intake_run_decision(intake: &AutonomousCodingIssueIntakeOutcome) -> Result<()> {
    match intake.decision {
        AutonomousCodingIssueIntakeDecision::NeedsAuthority
        | AutonomousCodingIssueIntakeDecision::ReadyToRun => Ok(()),
        AutonomousCodingIssueIntakeDecision::NeedsClarification => {
            bail!("intake-run refuses needs_clarification intake")
        }
        AutonomousCodingIssueIntakeDecision::SplitRequired => {
            bail!("intake-run refuses split_required intake")
        }
        AutonomousCodingIssueIntakeDecision::BlockedUnsafe => {
            bail!("intake-run refuses blocked_unsafe intake")
        }
        AutonomousCodingIssueIntakeDecision::MissingCredentials => {
            bail!("intake-run refuses missing_credentials intake")
        }
        AutonomousCodingIssueIntakeDecision::Unknown => {
            bail!("intake-run refuses unknown legacy intake")
        }
    }
}

fn intake_run_verifier_commands(
    intake: &AutonomousCodingIssueIntakeOutcome,
    explicit: &[String],
) -> Result<Vec<String>> {
    let mut commands = explicit
        .iter()
        .map(|command| command.trim())
        .filter(|command| !command.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if commands.is_empty() {
        commands = intake
            .verifier_plan
            .suggested_verifier_commands
            .iter()
            .map(|command| command.trim())
            .filter(|command| is_concrete_verifier_command(command))
            .map(str::to_string)
            .collect();
    }
    if commands.is_empty() {
        bail!("intake-run requires a concrete --verifier-command or persisted verifier command");
    }
    Ok(commands)
}

fn is_concrete_verifier_command(command: &str) -> bool {
    !command.is_empty() && !command.contains('<') && !command.contains('>')
}

fn provider_repair_policy(
    args: &ProviderRepairArgs,
) -> Result<AutonomousCodingProviderRepairPolicy> {
    if args.provider_repair_openrouter && args.provider_repair_command.is_some() {
        return Err(anyhow!(
            "--provider-repair-openrouter cannot be combined with --provider-repair-command"
        ));
    }

    if args.provider_repair_openrouter {
        let env_file = openrouter_repair_adapter::resolve_openrouter_env_file(
            args.provider_repair_env_file.clone(),
        );
        let max_attempts = if args.provider_repair_attempts == 0 {
            3
        } else {
            args.provider_repair_attempts
        };
        let model = openrouter_repair_adapter::resolve_openrouter_repair_model(
            args.provider_repair_model.as_deref(),
            env_file.as_deref(),
        );
        let mut adapter_args = vec![
            "openrouter-repair-adapter".to_string(),
            "--model".to_string(),
            model.clone(),
            "--request-timeout-ms".to_string(),
            args.provider_repair_timeout_ms.max(1).to_string(),
            "--max-tokens".to_string(),
            args.provider_repair_max_tokens.max(1).to_string(),
            "--max-retries".to_string(),
            args.provider_repair_max_retries.to_string(),
        ];
        if let Some(env_file) = env_file.as_ref() {
            adapter_args.push("--env-file".to_string());
            adapter_args.push(env_file.display().to_string());
        }
        if let Some(api_base) = args
            .provider_repair_api_base
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            adapter_args.push("--api-base".to_string());
            adapter_args.push(api_base.to_string());
        }
        return Ok(AutonomousCodingProviderRepairPolicy {
            enabled: true,
            max_attempts,
            command: Some(
                std::env::current_exe()
                    .unwrap_or_else(|_| PathBuf::from("tau_autonomous_coding_job")),
            ),
            args: adapter_args,
            provider: Some("openrouter".to_string()),
            model: Some(model),
        });
    }

    Ok(AutonomousCodingProviderRepairPolicy {
        enabled: args.provider_repair_command.is_some() && args.provider_repair_attempts > 0,
        max_attempts: args.provider_repair_attempts,
        command: args.provider_repair_command.clone(),
        args: args.provider_repair_args.clone(),
        provider: args.provider_repair_provider.clone(),
        model: args.provider_repair_model.clone(),
    })
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn github_env_from_process() -> std::collections::BTreeMap<String, String> {
    ["GH_TOKEN", "GITHUB_TOKEN"]
        .into_iter()
        .filter_map(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .map(|value| (key.to_string(), value))
        })
        .collect()
}

impl From<PrModeArg> for CodingMissionPrMode {
    fn from(value: PrModeArg) -> Self {
        match value {
            PrModeArg::Disabled => Self::Disabled,
            PrModeArg::PrReady => Self::PrReady,
            PrModeArg::Draft => Self::Draft,
        }
    }
}

impl From<MergeMethodArg> for AutonomousCodingMergeMethod {
    fn from(value: MergeMethodArg) -> Self {
        match value {
            MergeMethodArg::Merge => Self::Merge,
            MergeMethodArg::Squash => Self::Squash,
            MergeMethodArg::Rebase => Self::Rebase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tau_runtime::{
        AutonomousCodingIssueClarifyingQuestion, AutonomousCodingIssueIntakeClassification,
        AutonomousCodingIssueIntakeStatus, AutonomousCodingVerifierPlan,
    };

    #[test]
    fn spec_3809_intake_run_uses_concrete_persisted_verifier() {
        let intake = intake_fixture(
            AutonomousCodingIssueIntakeDecision::NeedsAuthority,
            "Fix concrete verifier",
            "Issue body",
            vec!["cargo test -p fixture-cli spec_3809_intake_run"],
        );

        let commands = intake_run_verifier_commands(&intake, &[]).expect("verifier commands");

        assert_eq!(
            commands,
            vec!["cargo test -p fixture-cli spec_3809_intake_run"]
        );
    }

    #[test]
    fn spec_3809_intake_run_uses_explicit_verifier_answer() {
        let intake = intake_fixture(
            AutonomousCodingIssueIntakeDecision::NeedsAuthority,
            "Fix missing verifier",
            "Issue body",
            vec!["cargo test -p <affected-crate> <focused-test>"],
        );

        let commands = intake_run_verifier_commands(
            &intake,
            &["cargo test -p tau-runtime spec_3809".to_string()],
        )
        .expect("explicit verifier");

        assert_eq!(commands, vec!["cargo test -p tau-runtime spec_3809"]);
    }

    #[test]
    fn spec_3809_intake_run_builds_issue_to_merge_request_from_persisted_intake() {
        let intake = intake_fixture(
            AutonomousCodingIssueIntakeDecision::NeedsAuthority,
            "Fix persisted issue",
            "Persisted issue body",
            vec!["cargo test -p tau-coding-agent spec_3809"],
        );
        let request = build_intake_run_request(
            intake,
            IssueIntakeRunArgs {
                runtime: RuntimeArgs {
                    state_dir: PathBuf::from(".tau/autonomous-coding"),
                    jobs_state_dir: PathBuf::from(".tau/jobs"),
                    runner_command: None,
                },
                provider_repair: provider_repair_args_fixture(),
                intake_id: "issue-3809-intake-run".to_string(),
                mission_id: None,
                session_key: "session-3809".to_string(),
                goal: None,
                branch_prefix: "codex/intake-run-".to_string(),
                verifier_commands: Vec::new(),
                allowed_roots: Vec::new(),
                edits: vec!["src/lib.rs=updated contents\n".to_string()],
                commit_message: None,
                pr_mode: PrModeArg::Draft,
                timeout_ms: Some(30_000),
                allow_auto_merge: false,
                merge_method: MergeMethodArg::Squash,
                delete_branch: false,
                gh_binary: None,
                started_unix_ms: Some(3_809),
            },
        )
        .expect("request");

        assert_eq!(request.intake_id, "issue-3809-intake-run");
        assert_eq!(request.mission_id, "issue-3809-intake-run-mission");
        assert_eq!(request.session_key, "session-3809");
        assert_eq!(request.repo_path, PathBuf::from("/tmp/tau-fixture"));
        assert_eq!(request.issue_title, "Fix persisted issue");
        assert_eq!(request.issue_body, "Persisted issue body");
        assert_eq!(request.goal, "Fix persisted issue\n\nPersisted issue body");
        assert_eq!(
            request.verifier_commands,
            vec!["cargo test -p tau-coding-agent spec_3809"]
        );
        assert_eq!(
            request.allowed_roots,
            vec![PathBuf::from("/tmp/tau-fixture")]
        );
        assert_eq!(request.pr_mode, CodingMissionPrMode::Draft);
        assert_eq!(request.controlled_edits.len(), 1);
        assert_eq!(
            request.controlled_edits[0].relative_path,
            PathBuf::from("src/lib.rs")
        );
        assert_eq!(request.controlled_edits[0].contents, "updated contents\n");
        assert_eq!(request.commit_message, "Resolve Fix persisted issue");
        assert_eq!(request.started_unix_ms, 3_809);
        assert!(!request.provider_repair.is_configured());
    }

    #[test]
    fn spec_3809_intake_run_rejects_vague_or_placeholder_intake() {
        let vague = intake_fixture(
            AutonomousCodingIssueIntakeDecision::NeedsClarification,
            "Fix it",
            "Broken",
            vec!["cargo test -p tau-runtime spec_3809"],
        );
        let vague_error = ensure_intake_run_decision(&vague).expect_err("vague refused");
        assert!(vague_error.to_string().contains("needs_clarification"));

        let placeholder = intake_fixture(
            AutonomousCodingIssueIntakeDecision::NeedsAuthority,
            "Fix placeholder",
            "Issue body",
            vec!["cargo test -p <affected-crate> <focused-test>"],
        );
        let placeholder_error =
            intake_run_verifier_commands(&placeholder, &[]).expect_err("placeholder refused");
        assert!(placeholder_error.to_string().contains("concrete"));
    }

    fn intake_fixture(
        decision: AutonomousCodingIssueIntakeDecision,
        title: &str,
        body: &str,
        verifier_commands: Vec<&str>,
    ) -> AutonomousCodingIssueIntakeOutcome {
        AutonomousCodingIssueIntakeOutcome {
            schema_version: 1,
            intake_id: "issue-3809-intake-run".to_string(),
            status: AutonomousCodingIssueIntakeStatus::Blocked,
            reason_code: "issue_intake_missing_edit_or_provider_authority".to_string(),
            classification:
                AutonomousCodingIssueIntakeClassification::MissingEditOrProviderAuthority,
            classification_summary: "fixture".to_string(),
            decision,
            clarifying_questions: vec![AutonomousCodingIssueClarifyingQuestion {
                reason_code: "mutation_authority".to_string(),
                question: "How should Tau mutate?".to_string(),
                required_input: "--edit or provider repair".to_string(),
            }],
            issue_url: "https://github.com/njfio/Tau/issues/3809".to_string(),
            issue_title: title.to_string(),
            issue_body: body.to_string(),
            issue_body_summary: body.to_string(),
            repo_path: PathBuf::from("/tmp/tau-fixture"),
            base_branch: "master".to_string(),
            required_authority: Vec::new(),
            verifier_plan: AutonomousCodingVerifierPlan {
                plan_kind: "rust".to_string(),
                summary: "fixture verifier".to_string(),
                suggested_verifier_commands: verifier_commands
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                missing_inputs: Vec::new(),
                next_action: "run intake".to_string(),
            },
            missing_inputs: Vec::new(),
            next_action_summary: "run intake".to_string(),
            created_unix_ms: 1,
            updated_unix_ms: 1,
        }
    }

    fn provider_repair_args_fixture() -> ProviderRepairArgs {
        ProviderRepairArgs {
            provider_repair_command: None,
            provider_repair_args: Vec::new(),
            provider_repair_attempts: 0,
            provider_repair_provider: None,
            provider_repair_model: None,
            provider_repair_openrouter: false,
            provider_repair_env_file: None,
            provider_repair_api_base: None,
            provider_repair_timeout_ms: 120_000,
            provider_repair_max_tokens: 1_200,
            provider_repair_max_retries: 1,
        }
    }
}
