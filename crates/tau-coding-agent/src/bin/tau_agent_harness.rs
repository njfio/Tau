//! Standalone binary: `tau-agent-harness`.
//!
//! Runs the canonical Tau autonomy benchmark fixture through the deterministic
//! mission harness and emits JSON proof.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use tau_agent_core::{
    load_autonomy_benchmark_fixture, run_autonomy_benchmark_fixture, MissionHarnessConfig,
};

#[derive(Debug, Parser)]
#[command(
    name = "tau-agent-harness",
    about = "Run Tau's autonomy benchmark fixture through the mission proof harness"
)]
struct Args {
    /// Benchmark fixture JSON. Defaults to the repo's canonical M334 fixture.
    #[arg(long)]
    fixture: Option<PathBuf>,

    /// Optional output file for the JSON proof. Stdout always receives JSON.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Root directory for mission learning memory writes.
    #[arg(long, default_value = ".tau/harness-memory")]
    memory_root: PathBuf,

    /// Stable run id for deterministic proof snapshots.
    #[arg(long, default_value = "local-harness-run")]
    run_id: String,

    /// Override proof timestamps for reproducible tests.
    #[arg(long)]
    started_unix_ms: Option<u64>,
}

fn main() -> ExitCode {
    let mut args = Args::parse();
    let fixture_path = args.fixture.take().unwrap_or_else(default_fixture_path);
    let started_unix_ms = args.started_unix_ms.unwrap_or_else(now_unix_ms);

    match run(args, fixture_path, started_unix_ms) {
        Ok(passed) if passed => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(2),
        Err(err) => {
            eprintln!("tau-agent-harness: {err:#}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Args, fixture_path: PathBuf, started_unix_ms: u64) -> anyhow::Result<bool> {
    let fixture = load_autonomy_benchmark_fixture(&fixture_path)?;
    let proof = run_autonomy_benchmark_fixture(
        &fixture,
        &MissionHarnessConfig {
            run_id: args.run_id,
            started_unix_ms,
            memory_root: args.memory_root,
            workspace_id: "tau-agent-harness".to_string(),
        },
    )?;
    let json = serde_json::to_string_pretty(&proof)?;
    if let Some(output) = args.output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, json.as_bytes())?;
    }
    println!("{json}");
    Ok(proof.passed)
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
