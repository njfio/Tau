#[tokio::test]
async fn spec_3810_c01_issue_to_merge_derives_repo_aware_rust_verifier() {
    let fixture = CodingJobFixture::new();
    fixture.install_rust_workspace_fixture("fail");
    let runtime = fixture.runtime_without_background();
    let provider = fixture.fake_provider_repair(
        r#"{"files":{"crates/fixture-cli/src/lib.rs":"pub fn marker() -> &'static str {\n    \"pass\"\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn spec_3810_repo_aware_verifier() {\n        assert_eq!(marker(), \"pass\");\n    }\n}\n"},"reason_code":"provider_rust_fix"}"#,
        0,
    );

    let outcome = runtime
        .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
            intake_id: "issue-3810-repo-aware-rust".to_string(),
            mission_id: "issue-3810-repo-aware-rust-mission".to_string(),
            session_key: "issue-3810-session".to_string(),
            repo_path: fixture.repo.path().to_path_buf(),
            issue_url: "https://github.com/njfio/Tau/issues/3810".to_string(),
            issue_title: "Fix fixture-cli Rust test `spec_3810_repo_aware_verifier`".to_string(),
            issue_body: "The fixture-cli crate has a failing Rust test named `spec_3810_repo_aware_verifier`; use the real package and focused test filter."
                .to_string(),
            goal: "Let Tau derive the focused Rust verifier from repo metadata.".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "codex/issue-to-merge-repo-aware-rust-".to_string(),
            verifier_commands: Vec::new(),
            pr_mode: CodingMissionPrMode::PrReady,
            allowed_roots: vec![fixture.repo.path().to_path_buf()],
            controlled_edits: Vec::new(),
            provider_repair: provider.policy(2, "fake-provider", "repair-model"),
            commit_message: "Fix repo-aware Rust verifier fixture".to_string(),
            timeout_ms: Some(10_000),
            allow_auto_merge: false,
            merge_method: AutonomousCodingMergeMethod::Squash,
            delete_branch: false,
            github_env: BTreeMap::new(),
            gh_binary: None,
            started_unix_ms: 14_000,
        })
        .await
        .expect("issue-to-merge repo-aware Rust verifier");

    assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::PrReady);
    let intake = outcome.intake.as_ref().expect("derived intake");
    assert_eq!(
        intake.decision,
        AutonomousCodingIssueIntakeDecision::ReadyToRun
    );
    assert_eq!(intake.verifier_plan.plan_kind, "rust");
    assert!(intake
        .verifier_plan
        .suggested_verifier_commands
        .iter()
        .any(|command| command == "cargo test -p fixture-cli spec_3810_repo_aware_verifier"));

    let submit = outcome.submit.as_ref().expect("submit");
    assert!(submit
        .record
        .verifier_commands
        .iter()
        .any(|command| command == "cargo test -p fixture-cli spec_3810_repo_aware_verifier"));
    let run = outcome.run.as_ref().expect("run");
    assert_eq!(run.status.status, AutonomousCodingJobStatus::PrReady);
    assert!(run
        .status
        .changed_files
        .iter()
        .any(|file| file == "crates/fixture-cli/src/lib.rs"));
}

#[tokio::test]
async fn spec_3810_c02_issue_to_merge_blocks_without_concrete_test_filter() {
    let fixture = CodingJobFixture::new();
    fixture.install_rust_workspace_fixture("fail");
    let runtime = fixture.runtime_without_background();
    let provider = fixture.fake_provider_repair(
        r#"{"files":{"crates/fixture-cli/src/lib.rs":"pub fn marker() -> &'static str {\n    \"pass\"\n}\n"},"reason_code":"provider_rust_fix"}"#,
        0,
    );

    let outcome = runtime
        .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
            intake_id: "issue-3810-missing-test-filter".to_string(),
            mission_id: "issue-3810-missing-test-filter-mission".to_string(),
            session_key: "issue-3810-session".to_string(),
            repo_path: fixture.repo.path().to_path_buf(),
            issue_url: "https://github.com/njfio/Tau/issues/3810".to_string(),
            issue_title: "Fix fixture-cli Rust failure".to_string(),
            issue_body:
                "The fixture-cli crate has a failing Rust test, but no exact test filter was provided."
                    .to_string(),
            goal: "Should not derive a vague Rust verifier.".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "codex/issue-to-merge-missing-filter-".to_string(),
            verifier_commands: Vec::new(),
            pr_mode: CodingMissionPrMode::PrReady,
            allowed_roots: vec![fixture.repo.path().to_path_buf()],
            controlled_edits: Vec::new(),
            provider_repair: provider.policy(2, "fake-provider", "repair-model"),
            commit_message: "Should not be used".to_string(),
            timeout_ms: Some(10_000),
            allow_auto_merge: false,
            merge_method: AutonomousCodingMergeMethod::Squash,
            delete_branch: false,
            github_env: BTreeMap::new(),
            gh_binary: None,
            started_unix_ms: 15_000,
        })
        .await
        .expect("issue-to-merge blocks without exact test filter");

    assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::Blocked);
    let intake = outcome.intake.as_ref().expect("blocked intake");
    assert_eq!(
        intake.classification,
        AutonomousCodingIssueIntakeClassification::MissingVerifier
    );
    assert!(intake
        .required_authority
        .iter()
        .any(|authority| authority.reason_code == "verifier_authority_required"));
    assert!(outcome.submit.is_none());
    assert!(outcome.run.is_none());
}

#[tokio::test]
async fn spec_3810_c03_issue_to_merge_blocks_when_package_is_not_in_metadata() {
    let fixture = CodingJobFixture::new();
    fixture.install_rust_workspace_fixture("fail");
    let runtime = fixture.runtime_without_background();
    let provider = fixture.fake_provider_repair(
        r#"{"files":{"crates/missing-crate/src/lib.rs":"pub fn marker() -> &'static str {\n    \"pass\"\n}\n"},"reason_code":"provider_rust_fix"}"#,
        0,
    );

    let outcome = runtime
        .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
            intake_id: "issue-3810-missing-package".to_string(),
            mission_id: "issue-3810-missing-package-mission".to_string(),
            session_key: "issue-3810-session".to_string(),
            repo_path: fixture.repo.path().to_path_buf(),
            issue_url: "https://github.com/njfio/Tau/issues/3810".to_string(),
            issue_title: "Fix missing-crate Rust test `spec_3810_repo_aware_verifier`"
                .to_string(),
            issue_body: "The missing-crate package has a failing Rust test named `spec_3810_repo_aware_verifier`, but that package is not in cargo metadata."
                .to_string(),
            goal: "Should not derive a verifier for an unresolved package.".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "codex/issue-to-merge-missing-package-".to_string(),
            verifier_commands: Vec::new(),
            pr_mode: CodingMissionPrMode::PrReady,
            allowed_roots: vec![fixture.repo.path().to_path_buf()],
            controlled_edits: Vec::new(),
            provider_repair: provider.policy(2, "fake-provider", "repair-model"),
            commit_message: "Should not be used".to_string(),
            timeout_ms: Some(10_000),
            allow_auto_merge: false,
            merge_method: AutonomousCodingMergeMethod::Squash,
            delete_branch: false,
            github_env: BTreeMap::new(),
            gh_binary: None,
            started_unix_ms: 16_000,
        })
        .await
        .expect("issue-to-merge blocks when package is not in metadata");

    assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::Blocked);
    let intake = outcome.intake.as_ref().expect("blocked intake");
    assert_eq!(
        intake.classification,
        AutonomousCodingIssueIntakeClassification::MissingVerifier
    );
    assert!(intake
        .required_authority
        .iter()
        .any(|authority| authority.reason_code == "verifier_authority_required"));
    assert!(outcome.submit.is_none());
    assert!(outcome.run.is_none());
}

#[tokio::test]
async fn spec_3811_c01_missing_filter_plan_names_exact_test_filter_input() {
    let fixture = CodingJobFixture::new();
    fixture.install_rust_workspace_fixture("fail");
    let runtime = fixture.runtime_without_background();
    let provider = fixture.fake_provider_repair(
        r#"{"files":{"crates/fixture-cli/src/lib.rs":"pub fn marker() -> &'static str {\n    \"pass\"\n}\n"},"reason_code":"provider_rust_fix"}"#,
        0,
    );

    let outcome = runtime
        .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
            intake_id: "issue-3811-missing-filter-detail".to_string(),
            mission_id: "issue-3811-missing-filter-detail-mission".to_string(),
            session_key: "issue-3811-session".to_string(),
            repo_path: fixture.repo.path().to_path_buf(),
            issue_url: "https://github.com/njfio/Tau/issues/3811".to_string(),
            issue_title: "Fix fixture-cli Rust failure".to_string(),
            issue_body: "The fixture-cli crate has a failing Rust test, but the issue forgot to name the exact test filter."
                .to_string(),
            goal: "Should report the exact missing verifier ingredient.".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "codex/issue-to-merge-missing-filter-detail-".to_string(),
            verifier_commands: Vec::new(),
            pr_mode: CodingMissionPrMode::PrReady,
            allowed_roots: vec![fixture.repo.path().to_path_buf()],
            controlled_edits: Vec::new(),
            provider_repair: provider.policy(2, "fake-provider", "repair-model"),
            commit_message: "Should not be used".to_string(),
            timeout_ms: Some(10_000),
            allow_auto_merge: false,
            merge_method: AutonomousCodingMergeMethod::Squash,
            delete_branch: false,
            github_env: BTreeMap::new(),
            gh_binary: None,
            started_unix_ms: 17_000,
        })
        .await
        .expect("issue-to-merge blocks with missing filter detail");

    assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::Blocked);
    let intake = outcome.intake.as_ref().expect("blocked intake");
    assert!(intake.verifier_plan.missing_inputs.iter().any(|input| {
        input == "exact quoted/backticked safe test filter token containing `spec`, `test`, `::`, or starting with `regression_`"
    }));
    assert!(intake
        .verifier_plan
        .next_action
        .contains("quoted/backticked test filter"));
}

#[tokio::test]
async fn spec_3811_c02_missing_package_plan_names_real_package_input() {
    let fixture = CodingJobFixture::new();
    fixture.install_rust_workspace_fixture("fail");
    let runtime = fixture.runtime_without_background();
    let provider = fixture.fake_provider_repair(
        r#"{"files":{"crates/missing-crate/src/lib.rs":"pub fn marker() -> &'static str {\n    \"pass\"\n}\n"},"reason_code":"provider_rust_fix"}"#,
        0,
    );

    let outcome = runtime
        .run_issue_to_merge(AutonomousCodingIssueToMergeRequest {
            intake_id: "issue-3811-missing-package-detail".to_string(),
            mission_id: "issue-3811-missing-package-detail-mission".to_string(),
            session_key: "issue-3811-session".to_string(),
            repo_path: fixture.repo.path().to_path_buf(),
            issue_url: "https://github.com/njfio/Tau/issues/3811".to_string(),
            issue_title: "Fix missing-crate Rust test `spec_3810_repo_aware_verifier`"
                .to_string(),
            issue_body: "The missing-crate package has a failing Rust test named `spec_3810_repo_aware_verifier`, but that package is not in cargo metadata."
                .to_string(),
            goal: "Should report the unresolved package verifier ingredient.".to_string(),
            base_branch: "master".to_string(),
            branch_prefix: "codex/issue-to-merge-missing-package-detail-".to_string(),
            verifier_commands: Vec::new(),
            pr_mode: CodingMissionPrMode::PrReady,
            allowed_roots: vec![fixture.repo.path().to_path_buf()],
            controlled_edits: Vec::new(),
            provider_repair: provider.policy(2, "fake-provider", "repair-model"),
            commit_message: "Should not be used".to_string(),
            timeout_ms: Some(10_000),
            allow_auto_merge: false,
            merge_method: AutonomousCodingMergeMethod::Squash,
            delete_branch: false,
            github_env: BTreeMap::new(),
            gh_binary: None,
            started_unix_ms: 18_000,
        })
        .await
        .expect("issue-to-merge blocks with missing package detail");

    assert_eq!(outcome.status, AutonomousCodingIssueToMergeStatus::Blocked);
    let intake = outcome.intake.as_ref().expect("blocked intake");
    assert!(intake
        .verifier_plan
        .missing_inputs
        .iter()
        .any(|input| input == "actual Cargo package name present in this repository (available: fixture-cli)"));
    assert!(intake
        .verifier_plan
        .next_action
        .contains("existing Cargo package"));
}
