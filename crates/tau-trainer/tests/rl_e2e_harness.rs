use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tau_trainer::{
    export_rl_e2e_harness_artifact, run_deterministic_rl_e2e_harness, RlE2eHarnessConfig,
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

    let first_payload = fs::read_to_string(&first_export.path).expect("read first payload");
    let second_payload = fs::read_to_string(&second_export.path).expect("read second payload");
    assert_eq!(
        first_payload, second_payload,
        "payload must be deterministic"
    );

    let _ = fs::remove_dir_all(temp);
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
