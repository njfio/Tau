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

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;
use tau_agent_core::{CodingMissionControlledEdit, CodingMissionPrMode};
use tau_runtime::{
    AutonomousCodingJobReplayRequest, AutonomousCodingJobRuntime, AutonomousCodingJobRuntimeConfig,
    AutonomousCodingJobSubmitRequest,
};

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
}

#[derive(Debug, Parser)]
struct JobSubmitArgs {
    #[command(flatten)]
    runtime: RuntimeArgs,

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
    #[arg(long, value_enum, default_value = "pr-ready")]
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

#[derive(Debug, Clone, Parser)]
struct RuntimeArgs {
    #[arg(long, default_value = ".tau/autonomous-coding")]
    state_dir: PathBuf,
    #[arg(long, default_value = ".tau/jobs")]
    jobs_state_dir: PathBuf,
    #[arg(long)]
    runner_command: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PrModeArg {
    Disabled,
    #[value(name = "pr-ready")]
    PrReady,
    Draft,
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

impl From<PrModeArg> for CodingMissionPrMode {
    fn from(value: PrModeArg) -> Self {
        match value {
            PrModeArg::Disabled => Self::Disabled,
            PrModeArg::PrReady => Self::PrReady,
            PrModeArg::Draft => Self::Draft,
        }
    }
}
