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
use tau_provider::{build_provider_client, CodexCliClient, CodexCliConfig};

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

    /// Mock provider response used for deterministic provider parsing tests.
    #[arg(long)]
    mock_provider_response: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum HarnessMode {
    Success,
    Resume,
    Blocked,
    ProviderSuccess,
}

impl HarnessMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Resume => "resume",
            Self::Blocked => "blocked",
            Self::ProviderSuccess => "provider_success",
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

#[derive(Debug, Serialize)]
struct ProviderProofReport {
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
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProviderUsageReport {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    cached_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct ProviderEditPayload {
    relative_path: String,
    contents: String,
    #[serde(default)]
    reason_code: Option<String>,
}

struct ProviderEditResolution {
    edit: CodingMissionControlledEdit,
    report: ProviderProofReport,
}

enum ProviderEditOutcome {
    Ready(ProviderEditResolution),
    Failed(ProviderProofReport),
}

struct ProviderFailureReportInput<'a> {
    mode: &'static str,
    model_ref: &'a ModelRef,
    dispatched: bool,
    reason_code: &'a str,
    error: Option<String>,
    response_text_bytes: Option<usize>,
    response_text_sha256: Option<String>,
    usage: Option<ProviderUsageReport>,
}

struct LiveLoopReportInput<'a> {
    benchmark_id: &'a str,
    task_id: &'a str,
    mode: HarnessMode,
    state: &'a CodingMissionState,
    outcome: &'a CodingMissionRunOutcome,
    resume: Option<ResumeReport>,
    pr_ready: Option<&'a CodingMissionPrReadyBundle>,
    provider: Option<ProviderProofReport>,
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
        HarnessMode::Success | HarnessMode::Resume | HarnessMode::ProviderSuccess => {
            "grep -q pass status.txt"
        }
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
    let mut provider_report = None;
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
            &mut provider_report,
            started_unix_ms,
        )?,
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
    let report = build_report(LiveLoopReportInput {
        benchmark_id: &fixture.benchmark_id,
        task_id: &task.id,
        mode: args.mode,
        state: &state,
        outcome: &outcome,
        resume: resume_report,
        pr_ready: pr_ready.as_ref(),
        provider: provider_report,
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

fn run_provider_success_case(
    runner: &CodingMissionRunner,
    state: &mut CodingMissionState,
    args: &Args,
    mission_id: &str,
    provider_report: &mut Option<ProviderProofReport>,
    started_unix_ms: u64,
) -> anyhow::Result<CodingMissionRunOutcome> {
    let red = runner
        .run(
            state,
            CodingMissionRunRequest {
                controlled_edit: None,
                commit_message: "Make M334 live verifier green with provider edit".to_string(),
                started_unix_ms: started_unix_ms + 1_000,
            },
        )
        .context("run provider-backed red checkpoint")?;
    if red.blocked_reason.is_some() || state.phase == CodingMissionPhase::Blocked {
        return Ok(red);
    }

    match resolve_provider_edit(args, state)
        .with_context(|| format!("resolve provider edit for {mission_id}"))?
    {
        ProviderEditOutcome::Ready(resolved) => {
            *provider_report = Some(resolved.report);
            let resumed = runner
                .resume(CodingMissionResumeRequest {
                    state_root: state.state_root.clone(),
                    mission_id: state.mission_id.clone(),
                    controlled_edit: Some(resolved.edit),
                    commit_message: "Make M334 live verifier green with provider edit".to_string(),
                    started_unix_ms: started_unix_ms + 2_000,
                    stop_after: None,
                })
                .context("resume provider-backed coding mission")?;
            *state = resumed.state;
            Ok(resumed.run)
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
                    started_unix_ms + 2_000,
                )
                .context("record provider-backed blocked state")?;
            *provider_report = Some(report);
            Ok(CodingMissionRunOutcome {
                phase: state.phase,
                verifier_passed: false,
                committed: None,
                blocked_reason: Some(blocked_reason),
                iterations: red.iterations,
            })
        }
    }
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
) -> anyhow::Result<ProviderEditOutcome> {
    let model_ref = ModelRef::parse(&args.provider_model)
        .with_context(|| format!("parse provider model '{}'", args.provider_model))?;

    if let Some(mock_response) = args.mock_provider_response.as_ref() {
        return Ok(provider_edit_from_response(
            "mock",
            &model_ref,
            true,
            None,
            None,
            mock_response,
        ));
    }

    let client = match provider_client(args, &model_ref) {
        Ok(client) => client,
        Err(error) => {
            return Ok(ProviderEditOutcome::Failed(provider_failure_report(
                ProviderFailureReportInput {
                    mode: "live",
                    model_ref: &model_ref,
                    dispatched: false,
                    reason_code: "provider_client_setup_failed",
                    error: Some(short_error(&error)),
                    response_text_bytes: None,
                    response_text_sha256: None,
                    usage: None,
                },
            )));
        }
    };

    let request = ChatRequest {
        model: model_ref.model.clone(),
        messages: vec![
            Message::system(
                "You generate one safe edit for a disposable Tau coding-loop proof. \
                 Return valid JSON only. No markdown, no prose.",
            ),
            Message::user(provider_prompt(state)?),
        ],
        tools: Vec::new(),
        tool_choice: None,
        json_mode: true,
        max_tokens: Some(256),
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
            Ok(provider_edit_from_response(
                "live",
                &model_ref,
                true,
                response.finish_reason,
                Some(response.usage),
                &text,
            ))
        }
        Err(error) => Ok(ProviderEditOutcome::Failed(provider_failure_report(
            ProviderFailureReportInput {
                mode: "live",
                model_ref: &model_ref,
                dispatched: true,
                reason_code: "provider_request_failed",
                error: Some(short_error(&error)),
                response_text_bytes: None,
                response_text_sha256: None,
                usage: None,
            },
        ))),
    }
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
    if auth_mode != "api-key" {
        cli_args.push("--provider-subscription-strict=true".to_string());
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

fn provider_prompt(state: &CodingMissionState) -> anyhow::Result<String> {
    let status_path = state.repo_path.join("status.txt");
    let current_status = fs::read_to_string(&status_path)
        .with_context(|| format!("read fixture status {}", status_path.display()))?;
    Ok(format!(
        "Goal:\n{}\n\nCurrent file status.txt contains:\n{}\n\nVerifier command:\n{}\n\nReturn exactly this JSON shape with the minimal edit needed to make the verifier pass:\n{{\"relative_path\":\"status.txt\",\"contents\":\"pass\\n\",\"reason_code\":\"provider_fix_for_live_loop\"}}",
        state.goal,
        current_status,
        state
            .verifier_commands
            .first()
            .map(String::as_str)
            .unwrap_or("grep -q pass status.txt")
    ))
}

fn provider_edit_from_response(
    mode: &'static str,
    model_ref: &ModelRef,
    dispatched: bool,
    finish_reason: Option<String>,
    usage: Option<ChatUsage>,
    response_text: &str,
) -> ProviderEditOutcome {
    let response_text_bytes = response_text.len();
    let response_text_sha256 = sha256_hex(response_text.as_bytes());
    let usage_report = usage.as_ref().map(provider_usage_report);
    match serde_json::from_str::<ProviderEditPayload>(response_text.trim()) {
        Ok(payload) => match provider_payload_to_edit(payload) {
            Ok(edit) => {
                let report = ProviderProofReport {
                    mode,
                    provider: model_ref.provider.as_str().to_string(),
                    model: model_ref.model.clone(),
                    dispatched,
                    parse_status: "parsed",
                    reason_code: "provider_edit_parsed".to_string(),
                    finish_reason,
                    usage: usage_report,
                    response_text_bytes: Some(response_text_bytes),
                    response_text_sha256: Some(response_text_sha256),
                    edit_relative_path: Some(edit.relative_path.display().to_string()),
                    edit_reason_code: Some(edit.reason_code.clone()),
                    error: None,
                };
                ProviderEditOutcome::Ready(ProviderEditResolution { edit, report })
            }
            Err(error) => {
                ProviderEditOutcome::Failed(provider_failure_report(ProviderFailureReportInput {
                    mode,
                    model_ref,
                    dispatched,
                    reason_code: "provider_edit_invalid",
                    error: Some(error),
                    response_text_bytes: Some(response_text_bytes),
                    response_text_sha256: Some(response_text_sha256),
                    usage: usage_report,
                }))
            }
        },
        Err(error) => {
            ProviderEditOutcome::Failed(provider_failure_report(ProviderFailureReportInput {
                mode,
                model_ref,
                dispatched,
                reason_code: "provider_output_malformed",
                error: Some(format!("provider response was not valid JSON: {error}")),
                response_text_bytes: Some(response_text_bytes),
                response_text_sha256: Some(response_text_sha256),
                usage: usage_report,
            }))
        }
    }
}

fn provider_payload_to_edit(
    payload: ProviderEditPayload,
) -> Result<CodingMissionControlledEdit, String> {
    let relative_path = PathBuf::from(payload.relative_path.trim());
    if relative_path.as_path() != Path::new("status.txt") {
        return Err(format!(
            "provider edit must target status.txt, got {}",
            relative_path.display()
        ));
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
    if contents.trim() != "pass" {
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

fn provider_failure_report(input: ProviderFailureReportInput<'_>) -> ProviderProofReport {
    ProviderProofReport {
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

fn build_report(input: LiveLoopReportInput<'_>) -> LiveLoopReport {
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
        HarnessMode::Success | HarnessMode::Resume | HarnessMode::ProviderSuccess => {
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
                && (input.mode != HarnessMode::ProviderSuccess
                    || input
                        .provider
                        .as_ref()
                        .map(|report| {
                            report.dispatched
                                && report.parse_status == "parsed"
                                && report.edit_relative_path.as_deref() == Some("status.txt")
                        })
                        .unwrap_or(false));
            if !ok {
                failure_reasons.push(match input.mode {
                    HarnessMode::ProviderSuccess => {
                        "provider_success_case_missing_required_evidence".to_string()
                    }
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
        provider: input.provider,
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
    use super::*;

    #[test]
    fn provider_backed_parses_json_edit() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response(
            "mock",
            &model_ref,
            true,
            Some("stop".to_string()),
            Some(ChatUsage {
                input_tokens: 10,
                output_tokens: 8,
                total_tokens: 18,
                cached_input_tokens: 2,
            }),
            r#"{"relative_path":"status.txt","contents":"pass","reason_code":"provider_test"}"#,
        );

        let ProviderEditOutcome::Ready(resolved) = outcome else {
            panic!("provider edit should parse");
        };
        assert_eq!(resolved.edit.relative_path, PathBuf::from("status.txt"));
        assert_eq!(resolved.edit.contents, "pass\n");
        assert_eq!(resolved.edit.reason_code, "provider_test");
        assert_eq!(resolved.report.parse_status, "parsed");
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
    fn provider_backed_malformed_response_fails_closed() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response("mock", &model_ref, true, None, None, "not-json");

        let ProviderEditOutcome::Failed(report) = outcome else {
            panic!("malformed provider output should fail");
        };
        assert_eq!(report.parse_status, "malformed");
        assert_eq!(report.reason_code, "provider_output_malformed");
        assert!(report.edit_relative_path.is_none());
        assert!(report.response_text_sha256.is_some());
    }

    #[test]
    fn provider_backed_rejects_path_escape() {
        let model_ref = ModelRef::parse("openai/test-model").expect("model parses");
        let outcome = provider_edit_from_response(
            "mock",
            &model_ref,
            true,
            None,
            None,
            r#"{"relative_path":"../status.txt","contents":"pass\n"}"#,
        );

        let ProviderEditOutcome::Failed(report) = outcome else {
            panic!("escaping provider edit should fail");
        };
        assert_eq!(report.parse_status, "invalid_edit");
        assert_eq!(report.reason_code, "provider_edit_invalid");
        assert!(report.edit_relative_path.is_none());
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
