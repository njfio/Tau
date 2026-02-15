//! Miscellaneous unit tests for shared argument normalization helpers.

use crate::{normalize_daemon_subcommand_args, normalize_startup_cli_args};

#[test]
fn unit_normalize_daemon_subcommand_args_maps_action_and_alias_flags() {
    let normalized = normalize_daemon_subcommand_args(vec![
        "tau-rs".to_string(),
        "daemon".to_string(),
        "status".to_string(),
        "--json".to_string(),
        "--state-dir".to_string(),
        ".tau/ops-daemon".to_string(),
    ]);
    assert_eq!(
        normalized,
        vec![
            "tau-rs",
            "--daemon-status",
            "--daemon-status-json",
            "--daemon-state-dir",
            ".tau/ops-daemon",
        ]
    );
}

#[test]
fn unit_normalize_startup_cli_args_maps_training_alias_equals_form_with_warning_snapshot() {
    let (normalized, warnings) = normalize_startup_cli_args(vec![
        "tau-rs".to_string(),
        "--train-config=.tau/train.json".to_string(),
    ]);
    assert_eq!(
        normalized,
        vec!["tau-rs", "--prompt-optimization-config=.tau/train.json"]
    );
    assert_eq!(
        warnings,
        vec![String::from(
            "deprecated CLI alias '--train-config' detected; use '--prompt-optimization-config' instead."
        )]
    );
}
