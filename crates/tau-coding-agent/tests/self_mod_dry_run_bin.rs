//! End-to-end test for the `self-mod-dry-run` operator binary.
//!
//! Exercises the argv → `run_dry_run_pipeline` → JSON-on-stdout contract so
//! downstream operators (and future autonomous call sites) can rely on a
//! stable external interface. Runs the real compiled binary via `assert_cmd`
//! against a `tempfile` workspace — no mocks.
//!
//! Covers three acceptance cases, one per policy branch:
//! 1. **Allowed skill target** — `skills/foo/manifest.toml` ⇒ `applied=false`,
//!    `safety_evaluation.allowed=true`, worktree path under the expected
//!    containment root, exit 0.
//! 2. **Denied source target** — `crates/x/src/main.rs` ⇒
//!    `safety_evaluation.allowed=false`, `blocked_by` contains
//!    `auto_apply_source_disabled`, exit 0 (pipeline ran cleanly; policy said no).
//! 3. **Hostile `--proposal-id` override** — `../escape` ⇒ non-zero exit,
//!    stderr mentions "proposal-id-override" rejection, no worktree created.

use std::process::Command;

use assert_cmd::cargo_bin;
use predicates::Predicate;
use predicates::str::contains;
use serde_json::Value;
use tempfile::TempDir;

fn bin() -> Command {
    Command::new(cargo_bin!("self_mod_dry_run"))
}

#[test]
fn dry_run_allows_skill_target_and_emits_containment_path() {
    let workspace = TempDir::new().unwrap();

    let output = bin()
        .args([
            "--target",
            "skills/foo/manifest.toml",
            "--workspace-root",
        ])
        .arg(workspace.path())
        .output()
        .expect("bin runs");

    assert!(
        output.status.success(),
        "exit={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("stdout is valid JSON");
    assert_eq!(json["applied"], Value::Bool(false));
    assert_eq!(json["safety_evaluation"]["allowed"], Value::Bool(true));
    assert_eq!(
        json["safety_evaluation"]["blocked_by"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );

    let worktree = json["worktree_path"]
        .as_str()
        .expect("worktree_path is a string");
    let expected_root = workspace
        .path()
        .join(".tau")
        .join("self-mod-worktrees");
    assert!(
        worktree.starts_with(expected_root.to_str().unwrap()),
        "worktree {worktree:?} must live under {expected_root:?}",
    );
}

#[test]
fn dry_run_denies_source_target_with_policy_blocker() {
    let workspace = TempDir::new().unwrap();

    let output = bin()
        .args([
            "--target",
            "crates/tau-ops/src/main.rs",
            "--workspace-root",
        ])
        .arg(workspace.path())
        .output()
        .expect("bin runs");

    assert!(
        output.status.success(),
        "exit={:?} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    let json: Value =
        serde_json::from_slice(&output.stdout).expect("stdout is valid JSON");
    assert_eq!(json["safety_evaluation"]["allowed"], Value::Bool(false));

    let blocked_by = json["safety_evaluation"]["blocked_by"]
        .as_array()
        .expect("blocked_by is an array");
    assert!(
        blocked_by
            .iter()
            .any(|v| v.as_str() == Some("auto_apply_source_disabled")),
        "expected `auto_apply_source_disabled` in blocked_by, got {blocked_by:?}",
    );
}

#[test]
fn dry_run_rejects_hostile_proposal_id_override_with_nonzero_exit() {
    let workspace = TempDir::new().unwrap();

    let output = bin()
        .args([
            "--target",
            "skills/foo.md",
            "--proposal-id",
            "../escape",
            "--workspace-root",
        ])
        .arg(workspace.path())
        .output()
        .expect("bin runs");

    assert!(
        !output.status.success(),
        "hostile override must fail; got exit={:?}",
        output.status.code(),
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        contains("proposal-id-override").eval(&stderr)
            && contains("rejected").eval(&stderr),
        "stderr should mention proposal-id-override rejection, got: {stderr}",
    );

    // Containment invariant: no worktree directory must exist for a rejected
    // override. We check the parent root rather than a specific id because the
    // rejection happens before any id is chosen.
    let worktrees_root = workspace
        .path()
        .join(".tau")
        .join("self-mod-worktrees");
    if worktrees_root.exists() {
        let entries: Vec<_> = std::fs::read_dir(&worktrees_root)
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(
            entries.is_empty(),
            "no worktree must be created for a rejected hostile override; found {entries:?}",
        );
    }
}
