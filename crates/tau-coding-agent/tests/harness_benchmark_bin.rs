use std::process::Command;

use assert_cmd::cargo_bin;
use serde_json::Value;
use tempfile::TempDir;

fn bin() -> Command {
    Command::new(cargo_bin!("tau_agent_harness"))
}

#[test]
fn harness_bin_runs_canonical_m334_benchmark_and_writes_proof() {
    let temp = TempDir::new().expect("tempdir");
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
    );
    let output_path = temp.path().join("proof.json");
    let memory_root = temp.path().join("memory");

    let output = bin()
        .args([
            "--fixture",
            fixture,
            "--output",
            output_path.to_str().expect("utf8 output"),
            "--memory-root",
            memory_root.to_str().expect("utf8 memory"),
            "--run-id",
            "bin-test-run",
        ])
        .output()
        .expect("bin runs");

    assert!(
        output.status.success(),
        "exit={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout: Value = serde_json::from_slice(&output.stdout).expect("stdout JSON");
    let file_json: Value =
        serde_json::from_slice(&std::fs::read(&output_path).expect("output file"))
            .expect("file JSON");
    assert_eq!(stdout, file_json);
    assert_eq!(stdout["benchmark_id"], "m334-tranche-one-autonomy");
    assert_eq!(stdout["passed"], true);
    assert_eq!(stdout["tasks"].as_array().map(Vec::len), Some(4));
    assert!(
        stdout["tasks"]
            .as_array()
            .expect("tasks array")
            .iter()
            .all(|task| task["passed"] == true
                && task["mission"]["status"] == "completed"
                && task["mission"]["final_learning_output"]["records"]
                    .as_array()
                    .is_some_and(|records| !records.is_empty())),
        "each task should include completed mission proof and learning output"
    );
}
