use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tau_trainer::{
    evaluate_rl_e2e_rollback_gate, export_rl_e2e_harness_artifact,
    run_deterministic_rl_e2e_harness, RlE2eCheck, RlE2eHarnessConfig,
};

#[tokio::test]
async fn spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries() {
    let temp = unique_temp_dir("spec-c03");
    let config = RlE2eHarnessConfig {
        run_id: "spec-c03".to_string(),
        output_dir: temp.join("artifacts"),
    };

    let first = run_deterministic_rl_e2e_harness(&config)
        .await
        .expect("first harness run");
    let first_export =
        export_rl_e2e_harness_artifact(&first, &config.output_dir).expect("export first artifact");
    let second = run_deterministic_rl_e2e_harness(&config)
        .await
        .expect("second harness run");
    let second_export = export_rl_e2e_harness_artifact(&second, &config.output_dir)
        .expect("export second artifact");

    assert!(first.pass);
    assert_eq!(first.schema_version, 1);
    assert_eq!(first.rollout_summary.total_rollouts, 6);
    assert_eq!(first.rollout_summary.succeeded, 6);
    assert!(first.gae_summary.advantages_len > 0);
    assert!(first.ppo_summary.optimizer_step_count > 0);
    assert!(
        first.promotion_gate.promotion_allowed,
        "promotion gate should pass for deterministic harness baseline"
    );
    assert!(
        !first.rollback_gate.rollback_required,
        "rollback gate should not require rollback when all checks pass"
    );

    let first_payload = fs::read_to_string(&first_export.path).expect("read first payload");
    let second_payload = fs::read_to_string(&second_export.path).expect("read second payload");
    assert_eq!(
        first_payload, second_payload,
        "payload must be deterministic"
    );
    let payload = serde_json::from_str::<serde_json::Value>(&first_payload).expect("valid json");
    assert!(payload["promotion_gate"].is_object());
    assert!(payload["rollback_gate"].is_object());
    assert!(payload["promotion_gate"]["reason_codes"].is_array());
    assert!(payload["rollback_gate"]["reason_codes"].is_array());

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn spec_c05_rl_e2e_rollback_gate_requires_rollback_when_gate_signals_failures() {
    let checks = vec![
        RlE2eCheck {
            id: "rollout_completion".to_string(),
            passed: true,
            detail: "ok".to_string(),
        },
        RlE2eCheck {
            id: "checkpoint_promotion_gate".to_string(),
            passed: false,
            detail: "blocked".to_string(),
        },
    ];

    let gate = evaluate_rl_e2e_rollback_gate(&checks, false, false);
    assert!(gate.rollback_required);
    assert!(gate
        .reason_codes
        .iter()
        .any(|code| code == "rollback_required_checkpoint_promotion_gate"));
    assert!(gate
        .reason_codes
        .iter()
        .any(|code| code == "rollback_required_policy_improvement_not_significant"));
}

#[test]
fn spec_c06_rl_e2e_rollback_gate_blocks_when_promotion_denied_without_failed_checks() {
    let checks = vec![RlE2eCheck {
        id: "rollout_completion".to_string(),
        passed: true,
        detail: "ok".to_string(),
    }];

    let gate = evaluate_rl_e2e_rollback_gate(&checks, false, true);
    assert!(gate.rollback_required);
    assert!(gate.failing_checks.is_empty());
    assert_eq!(
        gate.reason_codes,
        vec!["rollback_required_checkpoint_promotion_gate".to_string()]
    );
}

#[tokio::test]
async fn regression_spec_c04_rl_e2e_harness_rejects_output_path_that_is_a_file() {
    let temp = unique_temp_dir("spec-c04");
    let file_path = temp.join("not-a-directory.json");
    fs::write(&file_path, "{}").expect("write file");

    let config = RlE2eHarnessConfig {
        run_id: "spec-c04".to_string(),
        output_dir: file_path,
    };

    let error = run_deterministic_rl_e2e_harness(&config)
        .await
        .expect_err("file output path must fail");
    assert!(error.to_string().contains("output_dir"));
    assert!(error.to_string().contains("directory"));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn unit_rl_e2e_export_summary_uses_stable_filename_contract() {
    let filename = tau_trainer::rl_e2e_harness_filename("spec-c03");
    assert_eq!(
        filename,
        PathBuf::from("rl-e2e-harness-v1-spec-c03.json"),
        "filename contract changed unexpectedly"
    );
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "tau-rl-e2e-{label}-{suffix}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
