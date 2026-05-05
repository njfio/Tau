use std::path::Path;

use tau_agent_core::mission_harness::{
    load_autonomy_benchmark_fixture, run_autonomy_benchmark_fixture, run_harness_mission,
    MissionHarnessConfig,
};
use tau_agent_core::{MissionCompletionStatus, MissionLifecycleStatus, MissionVerifierStatus};
use tempfile::TempDir;

fn canonical_fixture_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
    ))
}

#[test]
fn harness_mission_completes_with_required_proof() {
    let memory = TempDir::new().expect("tempdir");
    let fixture = load_autonomy_benchmark_fixture(canonical_fixture_path()).expect("fixture loads");
    let task = fixture
        .tasks
        .iter()
        .find(|task| task.id == "repo_spec_to_pr_feature_delivery")
        .expect("task present");
    let config = MissionHarnessConfig {
        run_id: "test-run".to_string(),
        started_unix_ms: 1_000,
        memory_root: memory.path().to_path_buf(),
        workspace_id: "tau-test".to_string(),
    };

    let proof = run_harness_mission(&fixture, task, &config).expect("mission proof should run");

    assert!(proof.passed, "task proof should pass: {proof:?}");
    assert_eq!(proof.mission.status, MissionLifecycleStatus::Completed);
    assert_eq!(
        proof.mission.latest_completion.as_ref().map(|c| c.status),
        Some(MissionCompletionStatus::Success)
    );
    assert_eq!(
        proof.mission.latest_verifier.as_ref().map(|v| v.status),
        Some(MissionVerifierStatus::Passed)
    );
    assert!(proof.mission.ready_for_completion());
    assert!(
        proof
            .mission
            .verification_gates
            .iter()
            .any(|gate| gate.id == "memory_write_proof"
                && gate.status == Some(MissionVerifierStatus::Passed)),
        "memory write proof gate must pass"
    );
    assert!(
        proof
            .mission
            .final_learning_output
            .as_ref()
            .is_some_and(|output| !output.records.is_empty()),
        "final learning output must reference a persisted learning record"
    );
}

#[test]
fn autonomy_benchmark_fixture_runs_all_tasks_with_proof() {
    let memory = TempDir::new().expect("tempdir");
    let fixture = load_autonomy_benchmark_fixture(canonical_fixture_path()).expect("fixture loads");
    let config = MissionHarnessConfig {
        run_id: "suite-run".to_string(),
        started_unix_ms: 2_000,
        memory_root: memory.path().to_path_buf(),
        workspace_id: "tau-suite-test".to_string(),
    };

    let proof = run_autonomy_benchmark_fixture(&fixture, &config).expect("suite proof should run");

    assert!(proof.passed, "suite should pass: {proof:?}");
    assert_eq!(proof.benchmark_id, "m334-tranche-one-autonomy");
    assert_eq!(proof.tasks.len(), 4);
    assert_eq!(proof.failure_reasons, Vec::<String>::new());
    assert!(
        proof.tasks.iter().all(|task| task.passed
            && task.mission.status == MissionLifecycleStatus::Completed
            && task
                .operator_interventions_used
                .iter()
                .all(|intervention| fixture
                    .suite_policy
                    .allowed_operator_interventions
                    .contains(intervention))),
        "every task must complete without disallowed steering"
    );
}
