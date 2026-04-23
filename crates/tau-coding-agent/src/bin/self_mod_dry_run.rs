//! Standalone binary: `self-mod-dry-run`.
//!
//! Exercises the self-modification dry-run pipeline end-to-end so operators
//! can verify wiring and observe `tracing` output. Never mutates source code.
//!
//! Usage:
//!     self-mod-dry-run --target <path> [--proposal-id <id>] [--workspace-root <dir>]
//!
//! Exit codes:
//!     0  — pipeline succeeded (printed JSON result to stdout)
//!     2  — pipeline failed (error chain printed to stderr)
//!
//! The `tracing` subscriber level is controlled by the `TAU_SELF_MOD_LOG`
//! env var (e.g. `TAU_SELF_MOD_LOG=debug`); defaults to `info`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use tau_coding_agent::self_modification_pipeline::run_dry_run_pipeline;
use tau_coding_agent::self_modification_runtime::SelfModificationConfig;

#[derive(Debug, Parser)]
#[command(
    name = "self-mod-dry-run",
    about = "Run the self-modification dry-run pipeline end-to-end (no mutation)"
)]
struct Args {
    /// Path that would be modified. Classified via
    /// `classify_modification_target` to pick a policy bucket.
    #[arg(long)]
    target: String,

    /// Optional deterministic proposal id. Validated via
    /// `validate_proposal_id`; hostile values are rejected before any
    /// filesystem operation.
    #[arg(long)]
    proposal_id: Option<String>,

    /// Workspace root used as the containment anchor. Defaults to `.`.
    #[arg(long, default_value = ".")]
    workspace_root: PathBuf,
}

fn main() -> ExitCode {
    init_tracing();
    let args = Args::parse();

    let config = SelfModificationConfig::default();

    match run_dry_run_pipeline(
        &args.workspace_root,
        &args.target,
        args.proposal_id.as_deref(),
        &config,
    ) {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("self-mod-dry-run: failed to serialize result: {err}");
                ExitCode::from(2)
            }
        },
        Err(err) => {
            eprintln!("self-mod-dry-run: pipeline failed: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    // Default to silent so stdout remains pure JSON for consumers. Operators
    // opt in to logs via TAU_SELF_MOD_LOG (e.g. "info" or "debug"). Logs go
    // to stderr so they never corrupt the stdout JSON contract.
    let filter = EnvFilter::try_from_env("TAU_SELF_MOD_LOG")
        .unwrap_or_else(|_| EnvFilter::new("off"));
    let _ = fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}
