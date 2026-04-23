//! End-to-end library seam for the self-modification runtime.
//!
//! This module wires [`crate::self_modification_runtime`] primitives into a
//! single function — [`run_dry_run_pipeline`] — that classifies a target
//! path, applies the default policy, creates and then cleans up the
//! containment worktree, and returns a populated
//! [`SelfModificationResult`]. It is intentionally the *only* call site
//! that drives the whole flow: future drivers (CLI subcommand, LLM tool
//! handler, HTTP route, orchestrator callback) should call this function
//! rather than re-implement the sequencing.
//!
//! See ADR 0001 for the decision record.

use std::path::Path;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::self_modification_runtime::{
    can_auto_apply, classify_modification_target, cleanup_self_mod_worktree,
    create_self_mod_worktree, generate_proposal_id, validate_proposal_id, ModificationTarget,
    SelfModificationConfig, SelfModificationResult, SelfModificationSafetyResult,
};

/// Run the self-modification pipeline in dry-run mode.
///
/// No source-code mutation is performed. The function exists to prove the
/// wiring works end-to-end and to give later stages (real apply, LLM-driven
/// proposal synthesis, safety eval via `tau-safety`, rollback) a stable
/// seam to plug into.
///
/// Steps performed:
/// 1. [`validate_proposal_id`] on any caller-supplied override, else
///    [`generate_proposal_id`].
/// 2. [`classify_modification_target`] on `target_path`.
/// 3. Default-policy decision via [`can_auto_apply`].
/// 4. [`create_self_mod_worktree`] under
///    `<workspace_root>/.tau/self-mod-worktrees/<proposal_id>`.
/// 5. [`cleanup_self_mod_worktree`] to remove the worktree (dry-run does
///    not leave scratch state behind).
///
/// Returns a populated [`SelfModificationResult`] describing the decisions
/// taken. Any failure in steps (1), (4), or (5) is returned as a typed
/// error via `anyhow`.
pub fn run_dry_run_pipeline(
    workspace_root: &Path,
    target_path: &str,
    proposal_id_override: Option<&str>,
    config: &SelfModificationConfig,
) -> Result<SelfModificationResult> {
    let proposal_id = match proposal_id_override {
        Some(id) => {
            validate_proposal_id(id)
                .with_context(|| format!("proposal-id-override {:?} rejected", id))?;
            id.to_string()
        }
        None => generate_proposal_id(),
    };

    info!(
        proposal_id = %proposal_id,
        target = %target_path,
        workspace_root = %workspace_root.display(),
        "self-modification dry-run pipeline started",
    );

    let target = classify_modification_target(target_path);
    let auto_apply = can_auto_apply(config, &target);
    info!(
        proposal_id = %proposal_id,
        target = %target_path,
        classification = ?target,
        auto_apply,
        "self-modification target classified",
    );

    // Dry-run: we build the containment worktree to prove the create path
    // functions, then immediately clean it up.
    let worktree_path = create_self_mod_worktree(workspace_root, &proposal_id)
        .with_context(|| format!("failed to create worktree for proposal {:?}", proposal_id))?;

    // Cleanup is best-effort; a cleanup failure on a dry-run is a warning,
    // not a pipeline failure, because the containment-checked remove will
    // be tried again by operator tooling.
    if let Err(err) = cleanup_self_mod_worktree(workspace_root, &worktree_path) {
        warn!(
            proposal_id = %proposal_id,
            path = %worktree_path.display(),
            error = %err,
            "dry-run worktree cleanup reported error; operator tooling will retry",
        );
    }

    let result = SelfModificationResult {
        proposal_id: proposal_id.clone(),
        applied: false, // always false in dry-run
        safety_evaluation: SelfModificationSafetyResult {
            allowed: auto_apply_outcome_allowed(&target, auto_apply),
            blocked_by: blocked_by_for(&target, auto_apply),
            warnings: Vec::new(),
        },
        test_passed: None,
        rollback_available: false,
        worktree_path: Some(worktree_path),
        pr_url: None,
    };

    info!(
        proposal_id = %proposal_id,
        applied = result.applied,
        allowed = result.safety_evaluation.allowed,
        "self-modification dry-run pipeline finished",
    );
    Ok(result)
}

fn auto_apply_outcome_allowed(target: &ModificationTarget, auto_apply: bool) -> bool {
    // An "Other" target is always denied by policy; everything else inherits
    // the per-category auto-apply bit. This mirrors the default-closed stance
    // in `SelfModificationConfig::default`.
    match target {
        ModificationTarget::Other => false,
        _ => auto_apply,
    }
}

fn blocked_by_for(target: &ModificationTarget, auto_apply: bool) -> Vec<String> {
    if auto_apply_outcome_allowed(target, auto_apply) {
        return Vec::new();
    }
    match target {
        ModificationTarget::Other => vec!["target_classified_as_other".to_string()],
        ModificationTarget::Skill => vec!["auto_apply_skills_disabled".to_string()],
        ModificationTarget::Config => vec!["auto_apply_config_disabled".to_string()],
        ModificationTarget::Prompt => vec!["auto_apply_skills_disabled".to_string()],
        ModificationTarget::Source => vec!["auto_apply_source_disabled".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_happy_path_for_skill_target_under_default_config() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = SelfModificationConfig::default();
        let result = run_dry_run_pipeline(tmp.path(), "skills/foo/manifest.toml", None, &config)
            .expect("pipeline should succeed");

        assert!(!result.applied, "dry-run must never apply");
        assert!(result.safety_evaluation.allowed, "skills auto-apply by default");
        assert!(result.safety_evaluation.blocked_by.is_empty());
        assert!(result.worktree_path.is_some());
        // worktree must have been cleaned up
        let wt = result.worktree_path.as_ref().unwrap();
        assert!(!wt.exists(), "dry-run must not leave worktree behind");
    }

    #[test]
    fn dry_run_denies_source_edits_under_default_config() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = SelfModificationConfig::default();
        let result =
            run_dry_run_pipeline(tmp.path(), "crates/tau-ops/src/main.rs", None, &config).unwrap();

        assert!(!result.applied);
        assert!(!result.safety_evaluation.allowed);
        assert_eq!(
            result.safety_evaluation.blocked_by,
            vec!["auto_apply_source_disabled".to_string()]
        );
    }

    #[test]
    fn dry_run_classifies_hostile_target_path_as_other_and_denies() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = SelfModificationConfig::default();
        let result =
            run_dry_run_pipeline(tmp.path(), "../../etc/passwd", None, &config).unwrap();

        assert!(!result.applied);
        assert!(!result.safety_evaluation.allowed);
        assert_eq!(
            result.safety_evaluation.blocked_by,
            vec!["target_classified_as_other".to_string()]
        );
    }

    #[test]
    fn dry_run_rejects_hostile_proposal_id_override_before_any_fs_op() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = SelfModificationConfig::default();
        let err = run_dry_run_pipeline(
            tmp.path(),
            "skills/foo/manifest.toml",
            Some("../escape"),
            &config,
        )
        .expect_err("hostile proposal-id override must be rejected");
        let chain: Vec<String> = err.chain().map(|c| c.to_string()).collect();
        assert!(
            chain.iter().any(|m| m.contains("proposal-id-override")),
            "expected proposal-id-override rejection in error chain, got: {chain:?}",
        );
        // No worktree should be created under the workspace root for a
        // hostile id.
        let worktrees_root = tmp
            .path()
            .join(crate::self_modification_runtime::SELF_MOD_WORKTREES_SUBDIR);
        if worktrees_root.exists() {
            let entries: Vec<_> = std::fs::read_dir(&worktrees_root)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            assert!(
                entries.is_empty(),
                "hostile proposal-id must not create any worktree entries",
            );
        }
    }

    #[test]
    fn dry_run_accepts_explicit_safe_proposal_id_override() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = SelfModificationConfig::default();
        let result = run_dry_run_pipeline(
            tmp.path(),
            "skills/foo/manifest.toml",
            Some("deterministic-test-id-1"),
            &config,
        )
        .unwrap();
        assert_eq!(result.proposal_id, "deterministic-test-id-1");
    }

    #[test]
    fn dry_run_accepts_prompt_target_with_skills_auto_apply_enabled() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = SelfModificationConfig::default(); // skills default true
        let result =
            run_dry_run_pipeline(tmp.path(), "assets/template.hbs", None, &config).unwrap();
        assert!(result.safety_evaluation.allowed);
    }
}
