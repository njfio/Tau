//! Standalone binary: `tau-live-coding-loop-harness`.
//!
//! Drives the live `CodingMissionRunner` over a disposable repository for the
//! M334 repo-spec-to-PR benchmark task.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context};
use clap::{Parser, ValueEnum};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tau_agent_core::{
    load_autonomy_benchmark_fixture, CodingGitLifecycleEvidenceKind, CodingMissionConfig,
    CodingMissionControlledEdit, CodingMissionPhase, CodingMissionPrMode,
    CodingMissionPrPublicationStatus, CodingMissionPrReadyBundle, CodingMissionPrReadyRequest,
    CodingMissionResumeAction, CodingMissionResumeRequest, CodingMissionRunOutcome,
    CodingMissionRunRequest, CodingMissionRunner, CodingMissionState,
    CodingWorkspaceCommandEvidence, CodingWorkspaceCommandStatus,
};
use tau_ai::{ChatRequest, ChatUsage, LlmClient, Message, ModelRef, PromptCacheConfig, Provider};
use tau_cli::Cli;
use tau_provider::{
    build_provider_client, provider_api_key_candidates_with_inputs, resolve_api_key,
    CodexCliClient, CodexCliConfig,
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

    /// Provider/model ref for provider-backed proof mode.
    #[arg(long, default_value = "openai/gpt-4.1-mini")]
    provider_model: String,

    /// Optional provider API base override for provider-backed proof mode.
    #[arg(long)]
    provider_api_base: Option<String>,

    /// Provider auth mode for provider-backed proof mode.
    #[arg(long, default_value = "api-key")]
    provider_auth_mode: String,

    /// Request timeout for live provider-backed proof mode.
    #[arg(long, default_value_t = 120_000)]
    provider_timeout_ms: u64,

    /// Provider retries for live provider-backed proof mode.
    #[arg(long, default_value_t = 1)]
    provider_max_retries: usize,

    /// Provider output token cap for live provider-backed proof mode.
    #[arg(long, default_value_t = 1_024)]
    provider_max_tokens: u32,

    /// Additional provider repair attempts after the initial provider edit.
    #[arg(long, default_value_t = 2)]
    provider_repair_attempts: usize,

    /// Mock provider response for a specific provider attempt. Omitted attempts call the configured provider.
    #[arg(long = "mock-provider-response")]
    mock_provider_responses: Vec<String>,

    /// Verifier command(s) for real-repo mode. Repeat once per command.
    #[arg(long = "verifier-command")]
    verifier_commands: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum HarnessMode {
    Success,
    Resume,
    Blocked,
    ProviderSuccess,
    RealRepo,
}

impl HarnessMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Resume => "resume",
            Self::Blocked => "blocked",
            Self::ProviderSuccess => "provider_success",
            Self::RealRepo => "real_repo",
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
    provider: Option<ProviderProofReport>,
    provider_attempts: Vec<ProviderProofReport>,
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

#[derive(Debug, Clone, Serialize)]
struct ProviderProofReport {
    attempt_index: usize,
    mode: &'static str,
    provider: String,
    model: String,
    dispatched: bool,
    parse_status: &'static str,
    reason_code: String,
    finish_reason: Option<String>,
    usage: Option<ProviderUsageReport>,
    response_text_bytes: Option<usize>,
    response_text_sha256: Option<String>,
    edit_relative_path: Option<String>,
    edit_reason_code: Option<String>,
    repair_context_included: bool,
    failed_verifier_count: usize,
    diff_context_bytes: Option<usize>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderUsageReport {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    cached_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProviderEditPayload {
    Single(ProviderEditPayloadEntry),
    Multi {
        edits: Vec<ProviderEditPayloadEntry>,
    },
    FileMap {
        files: BTreeMap<String, String>,
        #[serde(default)]
        reason_code: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct ProviderEditPayloadEntry {
    relative_path: String,
    contents: String,
    #[serde(default)]
    reason_code: Option<String>,
}

struct ProviderEditResolution {
    edits: Vec<CodingMissionControlledEdit>,
    report: ProviderProofReport,
}

enum ProviderEditOutcome {
    Ready(ProviderEditResolution),
    Failed(ProviderProofReport),
}

#[derive(Debug, Clone, Default)]
struct ProviderRepairContext {
    failed_verifiers: Vec<ProviderVerifierFailureContext>,
    changed_files: Vec<String>,
    git_diff: String,
}

#[derive(Debug, Clone)]
struct ProviderVerifierFailureContext {
    argv: Vec<String>,
    exit_status: Option<i32>,
    stdout: String,
    stderr: String,
}

struct ProviderAttemptContext {
    attempt_index: usize,
    repair: Option<ProviderRepairContext>,
}

struct ProviderFailureReportInput<'a> {
    attempt_index: usize,
    mode: &'static str,
    model_ref: &'a ModelRef,
    dispatched: bool,
    reason_code: &'a str,
    error: Option<String>,
    response_text_bytes: Option<usize>,
    response_text_sha256: Option<String>,
    usage: Option<ProviderUsageReport>,
    repair: Option<&'a ProviderRepairContext>,
}

struct ProviderEditResponseInput<'a> {
    attempt_index: usize,
    mode: &'static str,
    model_ref: &'a ModelRef,
    repair: Option<&'a ProviderRepairContext>,
    dispatched: bool,
    finish_reason: Option<String>,
    usage: Option<ChatUsage>,
    response_text: &'a str,
}

struct LiveLoopReportInput<'a> {
    benchmark_id: &'a str,
    task_id: &'a str,
    mode: HarnessMode,
    state: &'a CodingMissionState,
    outcome: &'a CodingMissionRunOutcome,
    resume: Option<ResumeReport>,
    pr_ready: Option<&'a CodingMissionPrReadyBundle>,
    provider_attempts: Vec<ProviderProofReport>,
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
    if args.mode == HarnessMode::RealRepo {
        if !args.repo_root.join(".git").exists() {
            bail!("real-repo mode requires --repo-root to point at a git worktree");
        }
    } else {
        prepare_disposable_repo(&args.repo_root)?;
    }

    let mission_id = format!("{}-{}", args.run_id, args.mode.as_str());
    let verifier_commands = match args.mode {
        HarnessMode::RealRepo => {
            if args.verifier_commands.is_empty() {
                bail!("real-repo mode requires at least one --verifier-command");
            }
            args.verifier_commands.clone()
        }
        HarnessMode::Success | HarnessMode::Resume | HarnessMode::ProviderSuccess => {
            vec!["grep -q pass status.txt".to_string()]
        }
        HarnessMode::Blocked => vec!["definitely-not-a-tau-command".to_string()],
    };
    let base_branch = current_branch(&args.repo_root)
        .with_context(|| format!("resolve current branch for {}", args.repo_root.display()))?;
    let config = CodingMissionConfig {
        state_root: args.state_root.clone(),
        mission_id: mission_id.clone(),
        session_key: format!("session-{mission_id}"),
        repo_path: args.repo_root.clone(),
        issue_url: Some(match args.mode {
            HarnessMode::RealRepo => "https://github.com/njfio/Tau/issues/3792".to_string(),
            _ => "https://github.com/njfio/Tau/issues/3654".to_string(),
        }),
        goal: task.goal.clone(),
        base_branch,
        branch_prefix: match args.mode {
            HarnessMode::RealRepo => "codex/issue-3792-real-repo-harness".to_string(),
            _ => "codex/issue-3654-live-loop".to_string(),
        },
        verifier_commands,
        pr_mode: CodingMissionPrMode::PrReady,
        allowed_roots: vec![args.repo_root.clone(), args.state_root.clone()],
        created_unix_ms: started_unix_ms,
    };
    let mut state = CodingMissionState::create(config)?;
    let runner = CodingMissionRunner::new();

    let mut resume_report = None;
    let mut provider_attempts = Vec::new();
    let outcome = match args.mode {
        HarnessMode::Success => run_success_case(
            &runner,
            &mut state,
            pass_status_edit("controlled_fix_for_live_loop"),
            "Make M334 live verifier green",
            started_unix_ms,
        )?,
        HarnessMode::Resume => {
            let (outcome, resume) = run_resume_case(&runner, &mut state, started_unix_ms)?;
            resume_report = Some(resume);
            outcome
        }
        HarnessMode::Blocked => run_blocked_case(&runner, &mut state, started_unix_ms)?,
        HarnessMode::ProviderSuccess => run_provider_success_case(
            &runner,
            &mut state,
            &args,
            &mission_id,
            &mut provider_attempts,
            started_unix_ms,
        )?,
        HarnessMode::RealRepo => run_provider_success_case(
            &runner,
            &mut state,
            &args,
            &mission_id,
            &mut provider_attempts,
            started_unix_ms,
        )?,
    };
    let pr_ready = if state.phase == CodingMissionPhase::PrReady {
        let (title, risk_notes, rollback_notes) = match args.mode {
            HarnessMode::RealRepo => (
                "M334 real-repo autonomous coding harness".to_string(),
                vec!["Risk: temporary real repository worktree validation only".to_string()],
                vec!["Rollback: remove the harness branch/worktree".to_string()],
            ),
            _ => (
                "M334 live coding loop fixture".to_string(),
                vec!["Risk: disposable local fixture only".to_string()],
                vec!["Rollback: delete the disposable repo".to_string()],
            ),
        };
        Some(
            state
                .prepare_pr_ready_bundle(CodingMissionPrReadyRequest {
                    title: Some(title),
                    risk_notes,
                    rollback_notes,
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
    let report = build_report(LiveLoopReportInput {
        benchmark_id: &fixture.benchmark_id,
        task_id: &task.id,
        mode: args.mode,
        state: &state,
        outcome: &outcome,
        resume: resume_report,
        pr_ready: pr_ready.as_ref(),
        provider_attempts,
    });
    let passed = report.passed;
    write_report(&args.output, &report)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(passed)
}

fn run_success_case(
    runner: &CodingMissionRunner,
    state: &mut CodingMissionState,
    edit: CodingMissionControlledEdit,
    commit_message: &str,
    started_unix_ms: u64,
) -> anyhow::Result<CodingMissionRunOutcome> {
    runner
        .run(
            state,
            CodingMissionRunRequest {
                controlled_edit: Some(edit),
                controlled_edits: Vec::new(),
                commit_message: commit_message.to_string(),
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
                controlled_edits: Vec::new(),
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
            controlled_edits: Vec::new(),
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
                controlled_edits: Vec::new(),
                commit_message: "Blocked verifier should not commit".to_string(),
                started_unix_ms: started_unix_ms + 4_000,
            },
        )
        .context("run blocked coding mission")
}

fn run_provider_success_case(
    runner: &CodingMissionRunner,
    state: &mut CodingMissionState,
    args: &Args,
    mission_id: &str,
    provider_attempts: &mut Vec<ProviderProofReport>,
    started_unix_ms: u64,
) -> anyhow::Result<CodingMissionRunOutcome> {
    let red = runner
        .run(
            state,
            CodingMissionRunRequest {
                controlled_edit: None,
                controlled_edits: Vec::new(),
                commit_message: "Make M334 live verifier green with provider edit".to_string(),
                started_unix_ms: started_unix_ms + 1_000,
            },
        )
        .context("run provider-backed red checkpoint")?;
    if red.blocked_reason.is_some() || state.phase == CodingMissionPhase::Blocked {
        return Ok(red);
    }

    let total_provider_attempts = args.provider_repair_attempts.saturating_add(1).max(1);
    let mut latest_run = red;
    for attempt_index in 1..=total_provider_attempts {
        let repair = if attempt_index == 1 {
            None
        } else {
            Some(provider_repair_context(state)?)
        };
        let attempt = ProviderAttemptContext {
            attempt_index,
            repair,
        };
        match resolve_provider_edit(args, state, &attempt).with_context(|| {
            format!("resolve provider edit for {mission_id} attempt {attempt_index}")
        })? {
            ProviderEditOutcome::Ready(resolved) => {
                let ProviderEditResolution { edits, report } = resolved;
                provider_attempts.push(report);
                let resumed = runner
                    .resume(CodingMissionResumeRequest {
                        state_root: state.state_root.clone(),
                        mission_id: state.mission_id.clone(),
                        controlled_edit: None,
                        controlled_edits: edits,
                        commit_message: "Make M334 live verifier green with provider edit"
                            .to_string(),
                        started_unix_ms: started_unix_ms
                            .saturating_add(2_000)
                            .saturating_add((attempt_index as u64).saturating_mul(1_000)),
                        stop_after: None,
                    })
                    .context("resume provider-backed coding mission")?;
                *state = resumed.state;
                latest_run = resumed.run;
                if latest_run.verifier_passed || state.phase == CodingMissionPhase::PrReady {
                    return Ok(latest_run);
                }
                if latest_run.blocked_reason.is_some() || state.phase == CodingMissionPhase::Blocked
                {
                    return Ok(latest_run);
                }
            }
            ProviderEditOutcome::Failed(report) => {
                let blocked_reason = report.reason_code.clone();
                state
                    .transition_phase(
                        CodingMissionPhase::Blocked,
                        blocked_reason.as_str(),
                        report
                            .error
                            .clone()
                            .unwrap_or_else(|| blocked_reason.clone()),
                        started_unix_ms
                            .saturating_add(2_000)
                            .saturating_add((attempt_index as u64).saturating_mul(1_000)),
                    )
                    .context("record provider-backed blocked state")?;
                provider_attempts.push(report);
                return Ok(CodingMissionRunOutcome {
                    phase: state.phase,
                    verifier_passed: false,
                    committed: None,
                    blocked_reason: Some(blocked_reason),
                    iterations: latest_run.iterations,
                });
            }
        }
    }

    state
        .transition_phase(
            CodingMissionPhase::Blocked,
            "provider_repair_attempts_exhausted",
            "provider verifier repair attempts exhausted before verifiers passed",
            started_unix_ms.saturating_add(8_000),
        )
        .context("record exhausted provider repair state")?;
    Ok(CodingMissionRunOutcome {
        phase: state.phase,
        verifier_passed: false,
        committed: None,
        blocked_reason: Some("provider_repair_attempts_exhausted".to_string()),
        iterations: latest_run.iterations,
    })
}

fn pass_status_edit(reason_code: &str) -> CodingMissionControlledEdit {
    CodingMissionControlledEdit {
        relative_path: PathBuf::from("status.txt"),
        contents: "pass\n".to_string(),
        reason_code: reason_code.to_string(),
    }
}

fn resolve_provider_edit(
    args: &Args,
    state: &CodingMissionState,
    attempt: &ProviderAttemptContext,
) -> anyhow::Result<ProviderEditOutcome> {
    let model_ref = ModelRef::parse(&args.provider_model)
        .with_context(|| format!("parse provider model '{}'", args.provider_model))?;

    if let Some(mock_response) = mock_provider_response_for_attempt(args, attempt.attempt_index) {
        return Ok(provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: attempt.attempt_index,
            mode: "mock",
            model_ref: &model_ref,
            repair: attempt.repair.as_ref(),
            dispatched: true,
            finish_reason: None,
            usage: None,
            response_text: mock_response,
        }));
    }

    let client = match provider_client(args, &model_ref) {
        Ok(client) => client,
        Err(error) => {
            return Ok(ProviderEditOutcome::Failed(provider_failure_report(
                ProviderFailureReportInput {
                    attempt_index: attempt.attempt_index,
                    mode: "live",
                    model_ref: &model_ref,
                    dispatched: false,
                    reason_code: "provider_client_setup_failed",
                    error: Some(short_error(&error)),
                    response_text_bytes: None,
                    response_text_sha256: None,
                    usage: None,
                    repair: attempt.repair.as_ref(),
                },
            )));
        }
    };

    let request = ChatRequest {
        model: model_ref.model.clone(),
        messages: vec![
            Message::system(
                "You generate one complete, safe edit set for a Tau coding-loop proof. \
                 Return valid JSON only. No markdown, no prose. Include every file needed \
                 for the requested verifier pass in a single response.",
            ),
            Message::user(provider_prompt(state, attempt)?),
        ],
        tools: Vec::new(),
        tool_choice: None,
        json_mode: true,
        max_tokens: Some(args.provider_max_tokens.max(1)),
        temperature: Some(0.0),
        prompt_cache: PromptCacheConfig::default(),
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("build provider proof tokio runtime")?;
    match runtime.block_on(client.complete(request)) {
        Ok(response) => {
            let text = response.message.text_content();
            Ok(provider_edit_from_response(ProviderEditResponseInput {
                attempt_index: attempt.attempt_index,
                mode: "live",
                model_ref: &model_ref,
                repair: attempt.repair.as_ref(),
                dispatched: true,
                finish_reason: response.finish_reason,
                usage: Some(response.usage),
                response_text: &text,
            }))
        }
        Err(error) => Ok(ProviderEditOutcome::Failed(provider_failure_report(
            ProviderFailureReportInput {
                attempt_index: attempt.attempt_index,
                mode: "live",
                model_ref: &model_ref,
                dispatched: true,
                reason_code: "provider_request_failed",
                error: Some(short_error(&error)),
                response_text_bytes: None,
                response_text_sha256: None,
                usage: None,
                repair: attempt.repair.as_ref(),
            },
        ))),
    }
}

fn mock_provider_response_for_attempt(args: &Args, attempt_index: usize) -> Option<&str> {
    args.mock_provider_responses
        .get(attempt_index.saturating_sub(1))
        .map(String::as_str)
}

fn provider_client(args: &Args, model_ref: &ModelRef) -> anyhow::Result<Arc<dyn LlmClient>> {
    if args.provider_auth_mode.trim() == "codex-cli" {
        if model_ref.provider != Provider::OpenAi {
            bail!("codex-cli provider auth mode requires an openai/* provider model");
        }
        let client = CodexCliClient::new(CodexCliConfig {
            executable: "codex".to_string(),
            extra_args: Vec::new(),
            timeout_ms: args.provider_timeout_ms.max(1),
        })?;
        return Ok(Arc::new(client));
    }

    let cli = provider_cli(args, model_ref)?;
    build_provider_client(&cli, model_ref.provider)
}

fn provider_cli(args: &Args, model_ref: &ModelRef) -> anyhow::Result<Cli> {
    let auth_mode = args.provider_auth_mode.trim();
    if auth_mode.is_empty() {
        bail!("provider auth mode must not be empty");
    }
    let mut cli_args = vec![
        "tau-rs".to_string(),
        "--model".to_string(),
        args.provider_model.clone(),
        "--request-timeout-ms".to_string(),
        args.provider_timeout_ms.max(1).to_string(),
        "--provider-max-retries".to_string(),
        args.provider_max_retries.to_string(),
        "--provider-retry-budget-ms".to_string(),
        "0".to_string(),
    ];
    if !provider_auth_mode_is_api_key(auth_mode) {
        cli_args.push("--provider-subscription-strict=true".to_string());
    }
    if provider_auth_mode_is_api_key(auth_mode) {
        if let (Some(flag), Some(api_key)) = (
            provider_api_key_flag(model_ref.provider),
            provider_proof_api_key(model_ref.provider),
        ) {
            cli_args.push(flag.to_string());
            cli_args.push(api_key);
        }
    }
    match model_ref.provider {
        Provider::OpenAi | Provider::OpenRouter => {
            cli_args.push("--openai-auth-mode".to_string());
            cli_args.push(auth_mode.to_string());
            cli_args.push(format!(
                "--openai-codex-backend={}",
                if auth_mode == "api-key" {
                    "false"
                } else {
                    "true"
                }
            ));
        }
        Provider::Anthropic => {
            cli_args.push("--anthropic-auth-mode".to_string());
            cli_args.push(auth_mode.to_string());
            cli_args.push(format!(
                "--anthropic-claude-backend={}",
                if auth_mode == "api-key" {
                    "false"
                } else {
                    "true"
                }
            ));
        }
        Provider::Google => {
            cli_args.push("--google-auth-mode".to_string());
            cli_args.push(auth_mode.to_string());
            cli_args.push(format!(
                "--google-gemini-backend={}",
                if auth_mode == "api-key" {
                    "false"
                } else {
                    "true"
                }
            ));
        }
    }
    if let Some(api_base) = args.provider_api_base.as_ref() {
        let flag = match model_ref.provider {
            Provider::OpenAi | Provider::OpenRouter => "--api-base",
            Provider::Anthropic => "--anthropic-api-base",
            Provider::Google => "--google-api-base",
        };
        cli_args.push(flag.to_string());
        cli_args.push(api_base.clone());
    }
    Cli::try_parse_from(cli_args).context("parse provider proof CLI config")
}

fn provider_auth_mode_is_api_key(auth_mode: &str) -> bool {
    matches!(auth_mode.trim(), "api-key" | "api_key")
}

fn provider_api_key_flag(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::OpenAi | Provider::OpenRouter => Some("--openai-api-key"),
        Provider::Anthropic => Some("--anthropic-api-key"),
        Provider::Google => Some("--google-api-key"),
    }
}

fn provider_proof_api_key(provider: Provider) -> Option<String> {
    let candidates = provider_api_key_candidates_with_inputs(provider, None, None, None, None)
        .into_iter()
        .map(|(_source, value)| value)
        .collect();
    resolve_api_key(candidates)
}

fn provider_prompt(
    state: &CodingMissionState,
    attempt: &ProviderAttemptContext,
) -> anyhow::Result<String> {
    let repo_context = provider_repo_context(state)?;
    let repair_section = attempt
        .repair
        .as_ref()
        .map(render_provider_repair_context)
        .unwrap_or_default();
    Ok(format!(
        "Goal:\n{}\n\nRepository root:\n{}\n\nRepository context:\n{}\n\nVerifier commands:\n{}\n{}\n\nReturn valid JSON only. Return a complete edit set for all files needed in this single response; do not rely on later turns. Use either the legacy single-edit shape, this multi-edit shape:\n{{\"edits\":[{{\"relative_path\":\"path/from/repo/root\",\"contents\":\"file contents\",\"reason_code\":\"provider_fix\"}}]}}\nor this compact file-map shape:\n{{\"files\":{{\"path/from/repo/root\":\"file contents\"}},\"reason_code\":\"provider_fix\"}}",
        state.goal,
        state.repo_path.display(),
        repo_context,
        state
            .verifier_commands
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n"),
        repair_section
    ))
}

fn provider_repo_context(state: &CodingMissionState) -> anyhow::Result<String> {
    let status = git_lines(state.repo_path.as_path(), &["status", "--porcelain"])?;
    let files = git_lines(
        state.repo_path.as_path(),
        &["ls-files", "--cached", "--others", "--exclude-standard"],
    )?;
    let file_list = if files.trim().is_empty() {
        "none".to_string()
    } else {
        truncate_chars(files.trim(), 4_000)
    };
    let snippets = provider_repo_file_snippets(state.repo_path.as_path(), &files);
    Ok(format!(
        "Git status:\n{}\n\nTracked and untracked files:\n{}\n\nFile snippets:\n{}",
        if status.trim().is_empty() {
            "clean".to_string()
        } else {
            truncate_chars(status.trim(), 2_000)
        },
        file_list,
        snippets
    ))
}

fn provider_repo_file_snippets(repo_path: &Path, files: &str) -> String {
    let mut snippets = Vec::new();
    for relative in files
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with(".git/") && !line.starts_with(".tau/"))
        .take(12)
    {
        let path = Path::new(relative);
        if path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir
                        | std::path::Component::Prefix(_)
                        | std::path::Component::RootDir
                )
            })
        {
            continue;
        }
        let absolute = repo_path.join(path);
        if !absolute.is_file() {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&absolute) else {
            continue;
        };
        snippets.push(format!(
            "--- {} ---\n{}",
            relative,
            truncate_chars(redact_secret_like_tokens(&contents).as_str(), 1_600)
        ));
    }
    if snippets.is_empty() {
        "none".to_string()
    } else {
        snippets.join("\n\n")
    }
}

fn provider_repair_context(state: &CodingMissionState) -> anyhow::Result<ProviderRepairContext> {
    let failed_verifiers = state
        .command_evidence
        .iter()
        .rev()
        .filter(|evidence| {
            evidence.reason_code.starts_with("coding_verifier")
                && evidence.status == CodingWorkspaceCommandStatus::Failed
        })
        .take(4)
        .map(|evidence| ProviderVerifierFailureContext {
            argv: evidence.argv.clone(),
            exit_status: evidence.exit_status,
            stdout: read_artifact_snippet(evidence.stdout_path.as_path(), 2_000),
            stderr: read_artifact_snippet(evidence.stderr_path.as_path(), 4_000),
        })
        .collect::<Vec<_>>();
    let changed_files = git_lines(state.repo_path.as_path(), &["status", "--porcelain"])?;
    let git_diff = git_snippet(state.repo_path.as_path(), &["diff", "--", "."], 12_000)?;
    Ok(ProviderRepairContext {
        failed_verifiers,
        changed_files: parse_git_status_changed_file_names(&changed_files),
        git_diff,
    })
}

fn render_provider_repair_context(context: &ProviderRepairContext) -> String {
    let failed = context
        .failed_verifiers
        .iter()
        .enumerate()
        .map(|(index, failure)| {
            format!(
                "Failure {}:\nargv: {}\nexit_status: {}\nstdout:\n{}\nstderr:\n{}",
                index.saturating_add(1),
                failure.argv.join(" "),
                failure
                    .exit_status
                    .map(|status| status.to_string())
                    .unwrap_or_else(|| "spawn_failed".to_string()),
                failure.stdout,
                failure.stderr
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let changed_files = if context.changed_files.is_empty() {
        "none".to_string()
    } else {
        context.changed_files.join("\n")
    };
    format!(
        "\n\nRepair attempt context:\nThe previous provider edit did not pass verification. Return a targeted JSON patch for the current dirty worktree.\n\nFailed verifier evidence:\n{}\n\nChanged files:\n{}\n\nCurrent git diff:\n{}\n",
        failed,
        changed_files,
        context.git_diff
    )
}

fn read_artifact_snippet(path: &Path, max_chars: usize) -> String {
    let raw = fs::read_to_string(path).unwrap_or_default();
    truncate_chars(redact_secret_like_tokens(&raw).as_str(), max_chars)
}

fn git_lines(repo_path: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .with_context(|| format!("run git {} in {}", args.join(" "), repo_path.display()))?;
    if !output.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn git_snippet(repo_path: &Path, args: &[&str], max_chars: usize) -> anyhow::Result<String> {
    let raw = git_lines(repo_path, args)?;
    Ok(truncate_chars(
        redact_secret_like_tokens(&raw).as_str(),
        max_chars,
    ))
}

fn parse_git_status_changed_file_names(raw_status: &str) -> Vec<String> {
    raw_status
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.rsplit_once(" -> ")
                .map(|(_from, to)| to)
                .unwrap_or(line)
                .to_string()
        })
        .collect()
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("\n[truncated]");
    truncated
}

fn provider_edit_from_response(input: ProviderEditResponseInput<'_>) -> ProviderEditOutcome {
    let ProviderEditResponseInput {
        attempt_index,
        mode,
        model_ref,
        repair,
        dispatched,
        finish_reason,
        usage,
        response_text,
    } = input;
    let response_text_bytes = response_text.len();
    let response_text_sha256 = sha256_hex(response_text.as_bytes());
    let usage_report = usage.as_ref().map(provider_usage_report);
    match parse_provider_edit_payload(response_text) {
        Ok((payload, parse_status)) => match provider_payload_to_edits(payload) {
            Ok(edits) => {
                let edit_relative_path = provider_edit_paths_summary(&edits);
                let edit_reason_code = provider_edit_reason_summary(&edits);
                let report = ProviderProofReport {
                    attempt_index,
                    mode,
                    provider: model_ref.provider.as_str().to_string(),
                    model: model_ref.model.clone(),
                    dispatched,
                    parse_status,
                    reason_code: "provider_edit_parsed".to_string(),
                    finish_reason,
                    usage: usage_report,
                    response_text_bytes: Some(response_text_bytes),
                    response_text_sha256: Some(response_text_sha256),
                    edit_relative_path: Some(edit_relative_path),
                    edit_reason_code: Some(edit_reason_code),
                    repair_context_included: repair.is_some(),
                    failed_verifier_count: repair
                        .map(|context| context.failed_verifiers.len())
                        .unwrap_or_default(),
                    diff_context_bytes: repair.map(|context| context.git_diff.len()),
                    error: None,
                };
                ProviderEditOutcome::Ready(ProviderEditResolution { edits, report })
            }
            Err(error) => {
                ProviderEditOutcome::Failed(provider_failure_report(ProviderFailureReportInput {
                    attempt_index,
                    mode,
                    model_ref,
                    dispatched,
                    reason_code: "provider_edit_invalid",
                    error: Some(error),
                    response_text_bytes: Some(response_text_bytes),
                    response_text_sha256: Some(response_text_sha256),
                    usage: usage_report,
                    repair,
                }))
            }
        },
        Err(error) => {
            ProviderEditOutcome::Failed(provider_failure_report(ProviderFailureReportInput {
                attempt_index,
                mode,
                model_ref,
                dispatched,
                reason_code: "provider_output_malformed",
                error: Some(error),
                response_text_bytes: Some(response_text_bytes),
                response_text_sha256: Some(response_text_sha256),
                usage: usage_report,
                repair,
            }))
        }
    }
}

fn parse_provider_edit_payload(
    response_text: &str,
) -> Result<(ProviderEditPayload, &'static str), String> {
    let trimmed = response_text.trim();
    match serde_json::from_str::<ProviderEditPayload>(trimmed) {
        Ok(payload) => return Ok((payload, "parsed")),
        Err(error) => {
            let direct_error = error;
            for candidate in provider_json_candidates(trimmed) {
                if candidate.trim() == trimmed {
                    continue;
                }
                if let Ok(payload) = serde_json::from_str::<ProviderEditPayload>(candidate.trim()) {
                    return Ok((payload, "extracted_json"));
                }
            }
            Err(format!(
                "provider response was not valid JSON: {direct_error}"
            ))
        }
    }
}

fn provider_json_candidates(response_text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.extend(provider_fenced_json_candidates(response_text));
    if let Some(candidate) = first_balanced_json_object(response_text) {
        candidates.push(candidate);
    }
    candidates
}

fn provider_fenced_json_candidates(response_text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut offset = 0;
    while let Some(start_rel) = response_text[offset..].find("```") {
        let content_start = offset + start_rel + 3;
        let Some(end_rel) = response_text[content_start..].find("```") else {
            break;
        };
        let content_end = content_start + end_rel;
        let raw = &response_text[content_start..content_end];
        let candidate = raw
            .strip_prefix("json")
            .or_else(|| raw.strip_prefix("JSON"))
            .unwrap_or(raw)
            .trim_start_matches(['\r', '\n', ' ']);
        if !candidate.trim().is_empty() {
            candidates.push(candidate.trim().to_string());
        }
        offset = content_end + 3;
    }
    candidates
}

fn first_balanced_json_object(response_text: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in response_text.char_indices() {
        if start.is_none() {
            if ch == '{' {
                start = Some(index);
                depth = 1;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let start = start.expect("start set");
                    let end = index + ch.len_utf8();
                    return Some(response_text[start..end].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn provider_payload_to_edits(
    payload: ProviderEditPayload,
) -> Result<Vec<CodingMissionControlledEdit>, String> {
    let entries = match payload {
        ProviderEditPayload::Single(entry) => vec![entry],
        ProviderEditPayload::Multi { edits } => edits,
        ProviderEditPayload::FileMap { files, reason_code } => files
            .into_iter()
            .map(|(relative_path, contents)| ProviderEditPayloadEntry {
                relative_path,
                contents,
                reason_code: reason_code.clone(),
            })
            .collect(),
    };
    if entries.is_empty() {
        return Err("provider edit set must not be empty".to_string());
    }
    entries
        .into_iter()
        .map(provider_payload_entry_to_edit)
        .collect()
}

fn provider_payload_entry_to_edit(
    payload: ProviderEditPayloadEntry,
) -> Result<CodingMissionControlledEdit, String> {
    let relative_path = PathBuf::from(payload.relative_path.trim());
    if relative_path.as_os_str().is_empty() {
        return Err("provider edit path must not be empty".to_string());
    }
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
            )
        })
    {
        return Err("provider edit path escapes the disposable repo".to_string());
    }
    let mut contents = payload.contents;
    if relative_path.as_path() == Path::new("status.txt") && contents.trim() != "pass" {
        return Err("provider edit contents must make status.txt contain pass".to_string());
    }
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    let reason_code = payload
        .reason_code
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "provider_fix_for_live_loop".to_string());
    Ok(CodingMissionControlledEdit {
        relative_path,
        contents,
        reason_code,
    })
}

fn provider_edit_paths_summary(edits: &[CodingMissionControlledEdit]) -> String {
    edits
        .iter()
        .map(|edit| edit.relative_path.display().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn provider_edit_reason_summary(edits: &[CodingMissionControlledEdit]) -> String {
    edits
        .iter()
        .map(|edit| edit.reason_code.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn provider_failure_report(input: ProviderFailureReportInput<'_>) -> ProviderProofReport {
    ProviderProofReport {
        attempt_index: input.attempt_index,
        mode: input.mode,
        provider: input.model_ref.provider.as_str().to_string(),
        model: input.model_ref.model.clone(),
        dispatched: input.dispatched,
        parse_status: if input.reason_code == "provider_output_malformed" {
            "malformed"
        } else if input.reason_code == "provider_edit_invalid" {
            "invalid_edit"
        } else {
            "not_parsed"
        },
        reason_code: input.reason_code.to_string(),
        finish_reason: None,
        usage: input.usage,
        response_text_bytes: input.response_text_bytes,
        response_text_sha256: input.response_text_sha256,
        edit_relative_path: None,
        edit_reason_code: None,
        repair_context_included: input.repair.is_some(),
        failed_verifier_count: input
            .repair
            .map(|context| context.failed_verifiers.len())
            .unwrap_or_default(),
        diff_context_bytes: input.repair.map(|context| context.git_diff.len()),
        error: input.error,
    }
}

fn provider_usage_report(usage: &ChatUsage) -> ProviderUsageReport {
    ProviderUsageReport {
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        total_tokens: usage.total_tokens,
        cached_input_tokens: usage.cached_input_tokens,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn short_error(error: &impl std::fmt::Display) -> String {
    let rendered = redact_secret_like_tokens(&error.to_string());
    const MAX_ERROR_CHARS: usize = 600;
    if rendered.chars().count() <= MAX_ERROR_CHARS {
        return rendered;
    }
    let mut shortened = rendered.chars().take(MAX_ERROR_CHARS).collect::<String>();
    shortened.push_str("...");
    shortened
}

fn redact_secret_like_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| {
            if token.contains("sk-")
                || token.contains("sk-proj-")
                || token.contains("OPENAI_API_KEY")
                || token.contains("ANTHROPIC_API_KEY")
            {
                "[REDACTED_API_KEY]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn provider_report_satisfies_mode(mode: HarnessMode, report: Option<&ProviderProofReport>) -> bool {
    match mode {
        HarnessMode::ProviderSuccess => report
            .map(|report| {
                report.dispatched
                    && provider_parse_succeeded(report.parse_status)
                    && matches!(
                        report.edit_relative_path.as_deref(),
                        Some(paths) if paths.split(',').any(|path| path == "status.txt")
                    )
            })
            .unwrap_or(false),
        HarnessMode::RealRepo => report
            .map(|report| report.dispatched && provider_parse_succeeded(report.parse_status))
            .unwrap_or(false),
        HarnessMode::Success | HarnessMode::Resume | HarnessMode::Blocked => true,
    }
}

fn provider_parse_succeeded(parse_status: &str) -> bool {
    matches!(parse_status, "parsed" | "extracted_json")
}

fn build_report(input: LiveLoopReportInput<'_>) -> LiveLoopReport {
    let final_provider = input.provider_attempts.last().cloned();
    let commit = input
        .state
        .git_evidence
        .iter()
        .rev()
        .find(|evidence| evidence.kind == CodingGitLifecycleEvidenceKind::CommitCreated);
    let changed_files = input
        .pr_ready
        .map(|bundle| bundle.changed_files.clone())
        .or_else(|| commit.map(|evidence| evidence.changed_files.clone()))
        .unwrap_or_default();
    let branch = input
        .state
        .resume_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.branch_name.clone())
        .or_else(|| input.pr_ready.map(|bundle| bundle.branch_name.clone()))
        .or_else(|| commit.map(|evidence| evidence.branch_name.clone()))
        .unwrap_or_else(|| current_branch(&input.state.repo_path).unwrap_or_default());
    let verifier_transcript = input
        .state
        .command_evidence
        .iter()
        .filter(|evidence| evidence.reason_code.starts_with("coding_verifier"))
        .map(verifier_report)
        .collect::<Vec<_>>();
    let pr_ready_report = input.pr_ready.map(|bundle| PrReadyReport {
        status: pr_status_label(bundle.status),
        branch_name: bundle.branch_name.clone(),
        commit_hash: bundle.commit_hash.clone(),
        body_path: bundle.body_path.display().to_string(),
        manual_gh_pr_create_command: bundle.manual_gh_pr_create_command.clone(),
        pr_url: bundle.pr_url.clone(),
    });
    let mut failure_reasons = Vec::new();
    let passed = match input.mode {
        HarnessMode::Success
        | HarnessMode::Resume
        | HarnessMode::ProviderSuccess
        | HarnessMode::RealRepo => {
            let ok = input.state.phase == CodingMissionPhase::PrReady
                && pr_ready_report.is_some()
                && commit
                    .and_then(|evidence| evidence.commit_hash.as_ref())
                    .is_some()
                && verifier_transcript
                    .iter()
                    .any(|evidence| evidence.status == "failed")
                && verifier_transcript
                    .iter()
                    .any(|evidence| evidence.status == "succeeded")
                && provider_report_satisfies_mode(input.mode, final_provider.as_ref())
                && (input.mode != HarnessMode::RealRepo || changed_files.len() >= 2);
            if !ok {
                failure_reasons.push(match input.mode {
                    HarnessMode::ProviderSuccess => {
                        "provider_success_case_missing_required_evidence".to_string()
                    }
                    HarnessMode::RealRepo => "real_repo_case_missing_required_evidence".to_string(),
                    HarnessMode::Success | HarnessMode::Resume => {
                        "success_or_resume_case_missing_required_evidence".to_string()
                    }
                    HarnessMode::Blocked => unreachable!(),
                });
            }
            ok
        }
        HarnessMode::Blocked => {
            let ok = input.state.phase == CodingMissionPhase::Blocked
                && input.outcome.blocked_reason.as_deref()
                    == Some("verifier_command_failed_to_start")
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
        benchmark_id: input.benchmark_id.to_string(),
        task_id: input.task_id.to_string(),
        task_goal: input.state.goal.clone(),
        mode: input.mode.as_str().to_string(),
        passed,
        failure_reasons,
        mission_id: input.state.mission_id.clone(),
        session_key: input.state.session_key.clone(),
        phase: phase_label(input.state.phase),
        repo_path: input.state.repo_path.display().to_string(),
        state_root: input.state.state_root.display().to_string(),
        branch,
        commit_hash: commit.and_then(|evidence| evidence.commit_hash.clone()),
        changed_files,
        verifier_transcript,
        resume: input.resume,
        blocked_reason: input.outcome.blocked_reason.clone(),
        pr_ready: pr_ready_report,
        provider: final_provider,
        provider_attempts: input.provider_attempts,
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

#[cfg(test)]
mod provider_backed_tests {
    use std::sync::{Mutex, OnceLock};

    use super::*;
    use tempfile::tempdir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn restore_env(name: &str, prior: Option<String>) {
        match prior {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }
    }

    fn provider_test_args(provider_model: &str) -> Args {
        Args {
            fixture: None,
            task_id: "repo_spec_to_pr_feature_delivery".to_string(),
            mode: HarnessMode::ProviderSuccess,
            state_root: PathBuf::from("state"),
            repo_root: PathBuf::from("repo"),
            output: PathBuf::from("report.json"),
            run_id: "provider-test".to_string(),
            started_unix_ms: None,
            provider_model: provider_model.to_string(),
            provider_api_base: None,
            provider_auth_mode: "api-key".to_string(),
            provider_timeout_ms: 120_000,
            provider_max_retries: 1,
            provider_max_tokens: 1_024,
            provider_repair_attempts: 2,
            mock_provider_responses: Vec::new(),
            verifier_commands: Vec::new(),
        }
    }

    #[test]
    fn provider_backed_parses_json_edit() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: 1,
            mode: "mock",
            model_ref: &model_ref,
            repair: None,
            dispatched: true,
            finish_reason: Some("stop".to_string()),
            usage: Some(ChatUsage {
                input_tokens: 10,
                output_tokens: 8,
                total_tokens: 18,
                cached_input_tokens: 2,
            }),
            response_text: r#"{"relative_path":"status.txt","contents":"pass","reason_code":"provider_test"}"#,
        });

        let ProviderEditOutcome::Ready(resolved) = outcome else {
            panic!("provider edit should parse");
        };
        assert_eq!(resolved.edits.len(), 1);
        assert_eq!(resolved.edits[0].relative_path, PathBuf::from("status.txt"));
        assert_eq!(resolved.edits[0].contents, "pass\n");
        assert_eq!(resolved.edits[0].reason_code, "provider_test");
        assert_eq!(resolved.report.parse_status, "parsed");
        assert_eq!(resolved.report.attempt_index, 1);
        assert!(!resolved.report.repair_context_included);
        assert_eq!(
            resolved.report.edit_relative_path.as_deref(),
            Some("status.txt")
        );
        assert_eq!(
            resolved
                .report
                .usage
                .as_ref()
                .map(|usage| usage.total_tokens),
            Some(18)
        );
        assert!(resolved.report.response_text_sha256.is_some());
    }

    #[test]
    fn provider_backed_parses_json_edit_array() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let repair = repair_context_fixture();
        let outcome = provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: 2,
            mode: "mock",
            model_ref: &model_ref,
            repair: Some(&repair),
            dispatched: true,
            finish_reason: Some("stop".to_string()),
            usage: None,
            response_text: r#"{"edits":[{"relative_path":"status.txt","contents":"pass","reason_code":"provider_status"},{"relative_path":"docs/notes.txt","contents":"helper\n","reason_code":"provider_notes"}]}"#,
        });

        let ProviderEditOutcome::Ready(resolved) = outcome else {
            panic!("provider edit array should parse");
        };
        assert_eq!(resolved.edits.len(), 2);
        assert_eq!(resolved.edits[0].relative_path, PathBuf::from("status.txt"));
        assert_eq!(resolved.edits[0].contents, "pass\n");
        assert_eq!(
            resolved.edits[1].relative_path,
            PathBuf::from("docs/notes.txt")
        );
        assert_eq!(resolved.edits[1].contents, "helper\n");
        assert_eq!(resolved.report.parse_status, "parsed");
        assert_eq!(resolved.report.attempt_index, 2);
        assert!(resolved.report.repair_context_included);
        assert_eq!(resolved.report.failed_verifier_count, 1);
        assert_eq!(
            resolved.report.diff_context_bytes,
            Some(repair.git_diff.len())
        );
        assert_eq!(
            resolved.report.edit_relative_path.as_deref(),
            Some("status.txt,docs/notes.txt")
        );
    }

    #[test]
    fn provider_backed_extracts_fenced_json_edit_array() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: 1,
            mode: "mock",
            model_ref: &model_ref,
            repair: None,
            dispatched: true,
            finish_reason: Some("stop".to_string()),
            usage: None,
            response_text: "Here is the edit set:\n```json\n{\"edits\":[{\"relative_path\":\"index.html\",\"contents\":\"<!doctype html>\\n\",\"reason_code\":\"provider_index\"},{\"relative_path\":\"src/main.js\",\"contents\":\"console.log('ok');\\n\",\"reason_code\":\"provider_main\"}]}\n```",
        });

        let ProviderEditOutcome::Ready(resolved) = outcome else {
            panic!("provider fenced edit array should parse");
        };
        assert_eq!(resolved.report.parse_status, "extracted_json");
        assert_eq!(
            resolved.report.edit_relative_path.as_deref(),
            Some("index.html,src/main.js")
        );
        assert_eq!(resolved.edits.len(), 2);
        assert_eq!(resolved.edits[0].relative_path, PathBuf::from("index.html"));
        assert_eq!(
            resolved.edits[1].relative_path,
            PathBuf::from("src/main.js")
        );
    }

    #[test]
    fn provider_backed_parses_file_map_payload() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: 1,
            mode: "mock",
            model_ref: &model_ref,
            repair: None,
            dispatched: true,
            finish_reason: Some("stop".to_string()),
            usage: None,
            response_text: r#"{"files":{"index.html":"<!doctype html>\n","src/main.js":"console.log('ok');\n"},"reason_code":"provider_files"}"#,
        });

        let ProviderEditOutcome::Ready(resolved) = outcome else {
            panic!("provider file map should parse");
        };
        assert_eq!(resolved.report.parse_status, "parsed");
        assert_eq!(resolved.edits.len(), 2);
        assert_eq!(resolved.edits[0].reason_code, "provider_files");
        assert_eq!(resolved.edits[1].reason_code, "provider_files");
        assert_eq!(
            resolved.report.edit_relative_path.as_deref(),
            Some("index.html,src/main.js")
        );
    }

    #[test]
    fn provider_backed_extracted_json_satisfies_provider_modes() {
        let report = ProviderProofReport {
            attempt_index: 1,
            mode: "live",
            provider: "openrouter".to_string(),
            model: "qwen/qwen3.7-max".to_string(),
            dispatched: true,
            parse_status: "extracted_json",
            reason_code: "provider_edit_parsed".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: None,
            response_text_bytes: Some(128),
            response_text_sha256: Some("abc123".to_string()),
            edit_relative_path: Some("status.txt,src/main.js".to_string()),
            edit_reason_code: Some("provider_fix".to_string()),
            repair_context_included: false,
            failed_verifier_count: 0,
            diff_context_bytes: None,
            error: None,
        };

        assert!(provider_report_satisfies_mode(
            HarnessMode::ProviderSuccess,
            Some(&report)
        ));
        assert!(provider_report_satisfies_mode(
            HarnessMode::RealRepo,
            Some(&report)
        ));
    }

    #[test]
    fn provider_backed_malformed_response_fails_closed() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let repair = repair_context_fixture();
        let outcome = provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: 3,
            mode: "mock",
            model_ref: &model_ref,
            repair: Some(&repair),
            dispatched: true,
            finish_reason: None,
            usage: None,
            response_text: "not-json",
        });

        let ProviderEditOutcome::Failed(report) = outcome else {
            panic!("malformed provider output should fail");
        };
        assert_eq!(report.parse_status, "malformed");
        assert_eq!(report.reason_code, "provider_output_malformed");
        assert_eq!(report.attempt_index, 3);
        assert!(report.repair_context_included);
        assert!(report.edit_relative_path.is_none());
        assert!(report.response_text_sha256.is_some());
    }

    #[test]
    fn provider_backed_rejects_path_escape() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response(ProviderEditResponseInput {
            attempt_index: 1,
            mode: "mock",
            model_ref: &model_ref,
            repair: None,
            dispatched: true,
            finish_reason: None,
            usage: None,
            response_text: r#"{"relative_path":"../status.txt","contents":"pass\n"}"#,
        });

        let ProviderEditOutcome::Failed(report) = outcome else {
            panic!("escaping provider edit should fail");
        };
        assert_eq!(report.parse_status, "invalid_edit");
        assert_eq!(report.reason_code, "provider_edit_invalid");
        assert!(report.edit_relative_path.is_none());
    }

    #[test]
    fn provider_repair_context_render_includes_verifier_and_diff() {
        let rendered = render_provider_repair_context(&repair_context_fixture());

        assert!(rendered.contains("Repair attempt context"));
        assert!(rendered.contains("grep -q repaired notes.txt"));
        assert!(rendered.contains("expected repaired"));
        assert!(rendered.contains("status.txt"));
        assert!(rendered.contains("+pass"));
    }

    #[test]
    fn provider_prompt_includes_real_repo_context_and_complete_edit_instruction() {
        let temp = tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("src")).expect("repo dirs");
        std::process::Command::new("git")
            .args(["init", "-b", "master"])
            .current_dir(&repo)
            .status()
            .expect("git init");
        std::process::Command::new("git")
            .args(["config", "user.email", "provider-test@example.test"])
            .current_dir(&repo)
            .status()
            .expect("git config email");
        std::process::Command::new("git")
            .args(["config", "user.name", "Provider Test"])
            .current_dir(&repo)
            .status()
            .expect("git config name");
        fs::write(repo.join("README.md"), "existing readme\n").expect("readme");
        fs::write(repo.join("src/main.js"), "console.log('old');\n").expect("main");
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .status()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-m", "seed repo"])
            .current_dir(&repo)
            .status()
            .expect("git commit");

        let state = CodingMissionState::create(CodingMissionConfig {
            state_root: temp.path().join("state"),
            mission_id: "provider-prompt-alpha".to_string(),
            session_key: "session-alpha".to_string(),
            repo_path: repo,
            issue_url: None,
            goal: "Create index.html and update src/main.js".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "codex/".to_string(),
            verifier_commands: vec![
                "test -f index.html".to_string(),
                "grep -q Phaser src/main.js".to_string(),
            ],
            pr_mode: CodingMissionPrMode::Disabled,
            allowed_roots: vec![temp.path().to_path_buf()],
            created_unix_ms: 1_800_000_000_000,
        })
        .expect("create state");

        let prompt = provider_prompt(
            &state,
            &ProviderAttemptContext {
                attempt_index: 1,
                repair: None,
            },
        )
        .expect("provider prompt");

        assert!(prompt.contains("Repository context"));
        assert!(prompt.contains("Tracked and untracked files"));
        assert!(prompt.contains("README.md"));
        assert!(prompt.contains("--- src/main.js ---"));
        assert!(prompt.contains("console.log('old');"));
        assert!(prompt.contains("Return a complete edit set for all files needed"));
        assert!(prompt.contains("\"files\""));
        assert!(prompt.contains("grep -q Phaser src/main.js"));
    }

    fn repair_context_fixture() -> ProviderRepairContext {
        ProviderRepairContext {
            failed_verifiers: vec![ProviderVerifierFailureContext {
                argv: vec![
                    "grep".to_string(),
                    "-q".to_string(),
                    "repaired".to_string(),
                    "notes.txt".to_string(),
                ],
                exit_status: Some(1),
                stdout: String::new(),
                stderr: "expected repaired notes.txt".to_string(),
            }],
            changed_files: vec!["status.txt".to_string()],
            git_diff: "diff --git a/status.txt b/status.txt\n+pass\n".to_string(),
        }
    }

    fn run_provider_cli_stack_test(f: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .name("provider-cli-stack-test".to_string())
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("spawn provider cli stack test")
            .join()
            .expect("provider cli stack test should not panic");
    }

    #[test]
    fn provider_backed_openrouter_api_key_mode_injects_openrouter_key() {
        run_provider_cli_stack_test(|| {
            let _guard = env_lock().lock().expect("env lock");
            let prior_openrouter = std::env::var("OPENROUTER_API_KEY").ok();
            let prior_tau_openrouter = std::env::var("TAU_OPENROUTER_API_KEY").ok();
            let prior_openai = std::env::var("OPENAI_API_KEY").ok();

            std::env::set_var("OPENROUTER_API_KEY", "test-openrouter-key");
            std::env::remove_var("TAU_OPENROUTER_API_KEY");
            std::env::remove_var("OPENAI_API_KEY");

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let args = provider_test_args("openrouter/deepseek/deepseek-v4-flash");
                let model_ref = ModelRef::parse(&args.provider_model).expect("model parses");
                let cli = provider_cli(&args, &model_ref).expect("provider cli");

                assert_eq!(cli.openai_api_key.as_deref(), Some("test-openrouter-key"));
            }));

            restore_env("OPENROUTER_API_KEY", prior_openrouter);
            restore_env("TAU_OPENROUTER_API_KEY", prior_tau_openrouter);
            restore_env("OPENAI_API_KEY", prior_openai);
            result.expect("provider cli assertions should pass");
        });
    }

    #[test]
    fn provider_backed_redacts_key_shaped_error_tokens() {
        let redacted =
            short_error(&"provider returned 401 for sk-proj-abc123 and ANTHROPIC_API_KEY");

        assert!(redacted.contains("[REDACTED_API_KEY]"));
        assert!(!redacted.contains("sk-proj-abc123"));
        assert!(!redacted.contains("ANTHROPIC_API_KEY"));
    }
}
