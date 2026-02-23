use std::{env, path::PathBuf};

use tau_trainer::{
    export_rl_e2e_harness_artifact, run_deterministic_rl_e2e_harness, RlE2eHarnessConfig,
};

const HELP: &str = "\
tau-trainer RL end-to-end harness

Usage:
  cargo run -p tau-trainer --bin rl_e2e_harness -- [--run-id ID] [--output-dir PATH] [--print-json]

Options:
  --run-id ID        Stable run identifier used in exported artifact filename (default: deterministic)
  --output-dir PATH  Artifact output directory (default: artifacts/rl-e2e)
  --print-json       Print artifact JSON payload to stdout
  --help, -h         Show this help message
";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    run_id: String,
    output_dir: PathBuf,
    print_json: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            run_id: "deterministic".to_string(),
            output_dir: PathBuf::from("artifacts/rl-e2e"),
            print_json: false,
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut parsed = CliArgs::default();
    let mut it = args.into_iter();
    let _ = it.next();

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return Err(HELP.to_string()),
            "--run-id" => {
                let value = it.next().ok_or("missing value for --run-id")?;
                if value.trim().is_empty() {
                    return Err("--run-id must not be empty".to_string());
                }
                parsed.run_id = value;
            }
            "--output-dir" => {
                let value = it.next().ok_or("missing value for --output-dir")?;
                if value.trim().is_empty() {
                    return Err("--output-dir must not be empty".to_string());
                }
                parsed.output_dir = PathBuf::from(value);
            }
            "--print-json" => parsed.print_json = true,
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(parsed)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = match parse_args(env::args()) {
        Ok(args) => args,
        Err(err) if err == HELP => {
            println!("{HELP}");
            return;
        }
        Err(err) => {
            eprintln!("{err}");
            eprintln!();
            eprintln!("{HELP}");
            std::process::exit(2);
        }
    };

    let config = RlE2eHarnessConfig {
        run_id: args.run_id,
        output_dir: args.output_dir,
    };

    let artifact = match run_deterministic_rl_e2e_harness(&config).await {
        Ok(artifact) => artifact,
        Err(err) => {
            eprintln!("RL harness failed: {err:#}");
            std::process::exit(1);
        }
    };
    let export = match export_rl_e2e_harness_artifact(&artifact, &config.output_dir) {
        Ok(export) => export,
        Err(err) => {
            eprintln!("RL harness export failed: {err:#}");
            std::process::exit(1);
        }
    };

    println!(
        "rl e2e harness: pass={} rollouts={} path={}",
        artifact.pass,
        artifact.rollout_summary.total_rollouts,
        export.path.display()
    );
    if args.print_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&artifact.to_json_value()).expect("serialize RL e2e JSON")
        );
    }

    if !artifact.pass {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn unit_parse_args_defaults_are_stable() {
        let parsed = parse_args(vec!["rl_e2e_harness".to_string()]).expect("parse args");
        assert_eq!(parsed.run_id, "deterministic");
        assert_eq!(parsed.output_dir.to_string_lossy(), "artifacts/rl-e2e");
        assert!(!parsed.print_json);
    }

    #[test]
    fn functional_parse_args_supports_custom_values() {
        let parsed = parse_args(vec![
            "rl_e2e_harness".to_string(),
            "--run-id".to_string(),
            "demo".to_string(),
            "--output-dir".to_string(),
            "tmp/rl".to_string(),
            "--print-json".to_string(),
        ])
        .expect("parse args");
        assert_eq!(parsed.run_id, "demo");
        assert_eq!(parsed.output_dir.to_string_lossy(), "tmp/rl");
        assert!(parsed.print_json);
    }

    #[test]
    fn regression_parse_args_rejects_missing_run_id_value() {
        let err = parse_args(vec!["rl_e2e_harness".to_string(), "--run-id".to_string()])
            .expect_err("expected parse failure");
        assert!(err.contains("missing value for --run-id"));
    }
}
