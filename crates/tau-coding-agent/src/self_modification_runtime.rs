//! Self-modification runtime with worktree isolation.
//!
//! Provides configuration, classification, and worktree management for
//! autonomous source-code self-modification proposals. Safety evaluation
//! results are represented locally so this module can be used without
//! pulling in the full `tau-safety` crate at the type level.
//!
//! # Security model
//!
//! All filesystem operations are constrained to the subtree
//! `<workspace_root>/.tau/self-mod-worktrees/<proposal_id>`. `proposal_id`
//! values are validated against [`validate_proposal_id`] to reject path
//! traversal, absolute paths, path separators, and control characters
//! before being used to construct filesystem paths. `cleanup_self_mod_worktree`
//! additionally enforces that the path passed in lives inside the worktrees
//! root, so a hostile caller cannot trigger `remove_dir_all` on arbitrary
//! locations.
//!
//! # Callers
//!
//! See [`crate::self_modification_pipeline`] for the library seam that
//! exercises these primitives end-to-end, and the `self-mod-dry-run` bin
//! target for an operator-runnable demonstration.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Relative subtree (under the workspace root) that contains every
/// self-modification worktree. Kept as a single source of truth so
/// containment checks and path construction stay in sync.
pub const SELF_MOD_WORKTREES_SUBDIR: &str = ".tau/self-mod-worktrees";

/// Configuration for the self-modification runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationConfig {
    pub enabled: bool,
    pub auto_apply_skills: bool,
    pub auto_apply_config: bool,
    pub auto_apply_source: bool,
    pub workspace_root: PathBuf,
}

impl Default for SelfModificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_apply_skills: true,
            auto_apply_config: false,
            auto_apply_source: false,
            workspace_root: PathBuf::from("."),
        }
    }
}

/// Result of a self-modification attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationResult {
    pub proposal_id: String,
    pub applied: bool,
    pub safety_evaluation: SelfModificationSafetyResult,
    pub test_passed: Option<bool>,
    pub rollback_available: bool,
    pub worktree_path: Option<PathBuf>,
    pub pr_url: Option<String>,
}

/// Safety evaluation outcome for a self-modification proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationSafetyResult {
    pub allowed: bool,
    pub blocked_by: Vec<String>,
    pub warnings: Vec<String>,
}

/// Categorises a target path for self-modification policy decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModificationTarget {
    /// `skills/` directory entries.
    Skill,
    /// `.tau.toml` or other config files.
    Config,
    /// Prompt templates.
    Prompt,
    /// Rust source files under `crates/*/src/`.
    Source,
    /// Anything that does not match the categories above.
    Other,
}

/// Classify the modification target of a given path string.
///
/// Classification is based on path segments (not raw substring matches) so
/// that a harmless parent directory whose name happens to contain `prompt`
/// or `template` does not flip classification. The rules, evaluated in
/// order, are:
///
/// 1. Any segment equal to `skills`, or a path ending in `.md` with a
///    `skills` segment, classifies as [`ModificationTarget::Skill`].
/// 2. The exact filename `.tau.toml`, or a `.toml` file inside a `config`
///    segment, classifies as [`ModificationTarget::Config`].
/// 3. A segment equal to `prompts` or `templates` classifies as
///    [`ModificationTarget::Prompt`].
/// 4. A `.rs` file inside a `crates` segment classifies as
///    [`ModificationTarget::Source`].
/// 5. Everything else is [`ModificationTarget::Other`].
pub fn classify_modification_target(path: &str) -> ModificationTarget {
    // Normalise to forward slashes so the classifier behaves the same on
    // Windows-style inputs.
    let normalised = path.replace('\\', "/");
    let segments: Vec<&str> = normalised
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let last = segments.last().copied().unwrap_or("");

    let has_segment = |name: &str| segments.contains(&name);

    if segments.first().copied() == Some("skills")
        || (last.ends_with(".md") && has_segment("skills"))
    {
        return ModificationTarget::Skill;
    }
    if last == ".tau.toml" || (last.ends_with(".toml") && has_segment("config")) {
        return ModificationTarget::Config;
    }
    if has_segment("prompts") || has_segment("templates") {
        return ModificationTarget::Prompt;
    }
    // Filenames that *start* with `prompt` or `template` (e.g. `template.hbs`,
    // `prompt_v2.txt`) are treated as prompt/template assets even when they
    // don't live under a dedicated segment. Using the filename stem avoids
    // matching directories whose name merely contains the substring.
    let basename_stem = last.split('.').next().unwrap_or("");
    if basename_stem.starts_with("prompt") || basename_stem.starts_with("template") {
        return ModificationTarget::Prompt;
    }
    if last.ends_with(".rs") && has_segment("crates") {
        return ModificationTarget::Source;
    }
    ModificationTarget::Other
}

/// Determine whether auto-apply is permitted for the given target category.
pub fn can_auto_apply(config: &SelfModificationConfig, target: &ModificationTarget) -> bool {
    match target {
        ModificationTarget::Skill => config.auto_apply_skills,
        ModificationTarget::Config => config.auto_apply_config,
        ModificationTarget::Source => config.auto_apply_source,
        ModificationTarget::Prompt => config.auto_apply_skills, // same policy as skills
        ModificationTarget::Other => false,
    }
}

/// Validate a proposal identifier before it is used to construct filesystem
/// paths.
///
/// Rejects empty strings, path separators, relative traversal components,
/// absolute path markers, null bytes, and other ASCII control characters.
/// Accepts ASCII alphanumerics, `-`, `_`, and `.` (so `self-mod-<timestamp>`
/// identifiers produced by [`generate_proposal_id`] are always accepted).
///
/// # Errors
///
/// Returns [`std::io::ErrorKind::InvalidInput`] when the identifier would
/// allow a caller to escape the worktrees subtree or otherwise produce a
/// surprising filesystem path.
pub fn validate_proposal_id(proposal_id: &str) -> Result<(), std::io::Error> {
    if proposal_id.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "proposal_id must not be empty",
        ));
    }
    if proposal_id == "." || proposal_id == ".." {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "proposal_id must not be '.' or '..'",
        ));
    }
    for ch in proposal_id.chars() {
        let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.';
        if !ok {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "proposal_id contains disallowed character {:?}; allowed: [A-Za-z0-9._-]",
                    ch
                ),
            ));
        }
    }
    // Defence in depth: even though the character allow-list above excludes
    // separators, reject any identifier that looks like traversal.
    if proposal_id.contains("..") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "proposal_id must not contain '..'",
        ));
    }
    Ok(())
}

/// Compute the canonical worktrees root for a workspace.
fn worktrees_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(SELF_MOD_WORKTREES_SUBDIR)
}

/// Create a worktree directory for isolated source modification.
///
/// The worktree is always placed at
/// `<workspace_root>/.tau/self-mod-worktrees/<proposal_id>`. The
/// `proposal_id` is validated via [`validate_proposal_id`] before any path
/// is constructed, so a caller cannot supply `..` or absolute paths to
/// escape the containment root.
///
/// In production this would shell out to `git worktree add`; the current
/// implementation creates the directory structure so that later stages can
/// populate it.
pub fn create_self_mod_worktree(
    workspace_root: &Path,
    proposal_id: &str,
) -> Result<PathBuf, std::io::Error> {
    validate_proposal_id(proposal_id)?;
    let worktree_path = worktrees_root(workspace_root).join(proposal_id);
    std::fs::create_dir_all(&worktree_path)?;
    info!(
        proposal_id = %proposal_id,
        path = %worktree_path.display(),
        "self-modification worktree created",
    );
    Ok(worktree_path)
}

/// Remove a self-modification worktree that is no longer needed.
///
/// The `worktree_path` MUST point inside the canonical worktrees root
/// (`<workspace_root>/.tau/self-mod-worktrees/`). Anything else is rejected
/// with [`std::io::ErrorKind::InvalidInput`]. This prevents a caller from
/// reusing this helper to `remove_dir_all` outside the self-modification
/// subtree — the most dangerous failure mode of a self-improving loop.
///
/// Absent paths are treated as a successful no-op after logging, so
/// idempotent cleanup is still safe.
pub fn cleanup_self_mod_worktree(
    workspace_root: &Path,
    worktree_path: &Path,
) -> Result<(), std::io::Error> {
    let root = worktrees_root(workspace_root);

    // Canonicalise both sides when they exist so that symlinks and `./`
    // components cannot be used to sneak past the containment check. When
    // the target worktree is already gone, fall back to canonicalising its
    // parent and re-appending the final component — this preserves the
    // symlink resolution (important on macOS where `/tmp` resolves to
    // `/private/tmp`) while still allowing idempotent cleanup of an
    // already-removed worktree.
    let canonical_root = std::fs::canonicalize(&root).unwrap_or_else(|_| root.clone());
    let canonical_worktree = std::fs::canonicalize(worktree_path).unwrap_or_else(|_| {
        match (worktree_path.parent(), worktree_path.file_name()) {
            (Some(parent), Some(name)) => std::fs::canonicalize(parent)
                .map(|p| p.join(name))
                .unwrap_or_else(|_| worktree_path.to_path_buf()),
            _ => worktree_path.to_path_buf(),
        }
    });

    let inside_root =
        canonical_worktree.starts_with(&canonical_root) && canonical_worktree != canonical_root;
    if !inside_root {
        warn!(
            worktree_path = %worktree_path.display(),
            root = %root.display(),
            "refused to clean up worktree outside containment root",
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "worktree_path {:?} is not inside the self-mod worktrees root {:?}",
                worktree_path, root
            ),
        ));
    }

    if !worktree_path.exists() {
        debug!(
            path = %worktree_path.display(),
            "self-modification worktree already absent; cleanup is a no-op",
        );
        return Ok(());
    }

    std::fs::remove_dir_all(worktree_path)?;
    info!(
        path = %worktree_path.display(),
        "self-modification worktree removed",
    );
    Ok(())
}

/// Generate a unique proposal identifier based on the current timestamp.
pub fn generate_proposal_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("self-mod-{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // classify_modification_target
    // -----------------------------------------------------------------------

    #[test]
    fn classify_skills_directory_as_skill() {
        assert_eq!(
            classify_modification_target("skills/my-skill/manifest.toml"),
            ModificationTarget::Skill,
        );
    }

    #[test]
    fn classify_skills_markdown_as_skill() {
        assert_eq!(
            classify_modification_target("docs/skills/overview.md"),
            ModificationTarget::Skill,
        );
    }

    #[test]
    fn classify_tau_toml_as_config() {
        assert_eq!(
            classify_modification_target(".tau.toml"),
            ModificationTarget::Config,
        );
    }

    #[test]
    fn classify_config_toml_as_config() {
        assert_eq!(
            classify_modification_target("some/config/settings.toml"),
            ModificationTarget::Config,
        );
    }

    #[test]
    fn classify_prompt_file_as_prompt() {
        assert_eq!(
            classify_modification_target("prompts/system_prompt.txt"),
            ModificationTarget::Prompt,
        );
    }

    #[test]
    fn classify_template_file_as_prompt() {
        assert_eq!(
            classify_modification_target("assets/template.hbs"),
            ModificationTarget::Prompt,
        );
    }

    #[test]
    fn classify_rust_source_as_source() {
        assert_eq!(
            classify_modification_target("crates/tau-ops/src/main.rs"),
            ModificationTarget::Source,
        );
    }

    #[test]
    fn classify_unknown_path_as_other() {
        assert_eq!(
            classify_modification_target("README.txt"),
            ModificationTarget::Other,
        );
    }

    // -----------------------------------------------------------------------
    // can_auto_apply
    // -----------------------------------------------------------------------

    #[test]
    fn auto_apply_skill_when_enabled() {
        let config = SelfModificationConfig {
            auto_apply_skills: true,
            ..Default::default()
        };
        assert!(can_auto_apply(&config, &ModificationTarget::Skill));
    }

    #[test]
    fn auto_apply_skill_when_disabled() {
        let config = SelfModificationConfig {
            auto_apply_skills: false,
            ..Default::default()
        };
        assert!(!can_auto_apply(&config, &ModificationTarget::Skill));
    }

    #[test]
    fn auto_apply_config_when_enabled() {
        let config = SelfModificationConfig {
            auto_apply_config: true,
            ..Default::default()
        };
        assert!(can_auto_apply(&config, &ModificationTarget::Config));
    }

    #[test]
    fn auto_apply_config_when_disabled() {
        let config = SelfModificationConfig::default();
        assert!(!can_auto_apply(&config, &ModificationTarget::Config));
    }

    #[test]
    fn auto_apply_source_when_enabled() {
        let config = SelfModificationConfig {
            auto_apply_source: true,
            ..Default::default()
        };
        assert!(can_auto_apply(&config, &ModificationTarget::Source));
    }

    #[test]
    fn auto_apply_source_when_disabled() {
        let config = SelfModificationConfig::default();
        assert!(!can_auto_apply(&config, &ModificationTarget::Source));
    }

    #[test]
    fn auto_apply_prompt_follows_skill_policy() {
        let config = SelfModificationConfig {
            auto_apply_skills: true,
            ..Default::default()
        };
        assert!(can_auto_apply(&config, &ModificationTarget::Prompt));

        let config_off = SelfModificationConfig {
            auto_apply_skills: false,
            ..Default::default()
        };
        assert!(!can_auto_apply(&config_off, &ModificationTarget::Prompt));
    }

    #[test]
    fn auto_apply_other_is_always_false() {
        let config = SelfModificationConfig {
            auto_apply_skills: true,
            auto_apply_config: true,
            auto_apply_source: true,
            ..Default::default()
        };
        assert!(!can_auto_apply(&config, &ModificationTarget::Other));
    }

    // -----------------------------------------------------------------------
    // SelfModificationConfig::default
    // -----------------------------------------------------------------------

    #[test]
    fn default_config_has_correct_values() {
        let config = SelfModificationConfig::default();
        assert!(config.enabled);
        assert!(config.auto_apply_skills);
        assert!(!config.auto_apply_config);
        assert!(!config.auto_apply_source);
        assert_eq!(config.workspace_root, PathBuf::from("."));
    }

    // -----------------------------------------------------------------------
    // generate_proposal_id
    // -----------------------------------------------------------------------

    #[test]
    fn generate_proposal_id_has_expected_prefix() {
        let id = generate_proposal_id();
        assert!(!id.is_empty());
        assert!(
            id.starts_with("self-mod-"),
            "expected 'self-mod-' prefix, got: {id}"
        );
    }

    #[test]
    fn generate_proposal_id_produces_non_empty_suffix() {
        let id = generate_proposal_id();
        let suffix = id.strip_prefix("self-mod-").unwrap();
        assert!(!suffix.is_empty());
    }

    // -----------------------------------------------------------------------
    // create_self_mod_worktree
    // -----------------------------------------------------------------------

    #[test]
    fn create_self_mod_worktree_creates_directory() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let result = create_self_mod_worktree(tmp.path(), "test-proposal-1");
        assert!(result.is_ok());
        let worktree_path = result.unwrap();
        assert!(worktree_path.exists());
        assert!(worktree_path.is_dir());
        assert!(worktree_path.ends_with("test-proposal-1"));
    }

    // -----------------------------------------------------------------------
    // cleanup_self_mod_worktree
    // -----------------------------------------------------------------------

    #[test]
    fn cleanup_self_mod_worktree_removes_directory() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let worktree_path =
            create_self_mod_worktree(tmp.path(), "test-cleanup-1").expect("create should succeed");
        assert!(worktree_path.exists());

        let result = cleanup_self_mod_worktree(tmp.path(), &worktree_path);
        assert!(result.is_ok());
        assert!(!worktree_path.exists());
    }

    #[test]
    fn cleanup_self_mod_worktree_succeeds_when_path_does_not_exist_but_is_contained() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        // Construct an absent but *contained* worktree path.
        let absent = tmp
            .path()
            .join(SELF_MOD_WORKTREES_SUBDIR)
            .join("never-existed");
        assert!(!absent.exists());
        // Create the root so containment check resolves lexically.
        std::fs::create_dir_all(tmp.path().join(SELF_MOD_WORKTREES_SUBDIR)).unwrap();
        let result = cleanup_self_mod_worktree(tmp.path(), &absent);
        assert!(result.is_ok(), "absent but contained path should no-op");
    }

    #[test]
    fn cleanup_self_mod_worktree_refuses_path_outside_containment_root() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        // A directory outside the worktrees subtree — even inside the same
        // workspace root — must be refused.
        let outside = tmp.path().join("not-a-worktree");
        std::fs::create_dir_all(&outside).unwrap();

        let err = cleanup_self_mod_worktree(tmp.path(), &outside)
            .expect_err("cleanup should refuse paths outside containment");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(outside.exists(), "refused cleanup must NOT delete the path");
    }

    #[test]
    fn cleanup_self_mod_worktree_refuses_workspace_root_itself() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        // Passing the worktrees root itself must not trigger remove_dir_all
        // on the root — only strict descendants may be cleaned.
        let root = tmp.path().join(SELF_MOD_WORKTREES_SUBDIR);
        std::fs::create_dir_all(&root).unwrap();
        let err = cleanup_self_mod_worktree(tmp.path(), &root)
            .expect_err("cleanup should refuse the containment root itself");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(root.exists());
    }

    // -----------------------------------------------------------------------
    // validate_proposal_id — adversarial input hardening
    // -----------------------------------------------------------------------

    #[test]
    fn validate_proposal_id_accepts_generator_output() {
        let id = generate_proposal_id();
        assert!(validate_proposal_id(&id).is_ok());
    }

    #[test]
    fn validate_proposal_id_accepts_simple_identifiers() {
        for good in [
            "self-mod-12345",
            "proposal.v2",
            "abc_123",
            "A",
            "0",
            "self-mod-1700000000000",
        ] {
            assert!(
                validate_proposal_id(good).is_ok(),
                "expected {good:?} to be accepted"
            );
        }
    }

    #[test]
    fn validate_proposal_id_rejects_traversal_and_separators() {
        for bad in [
            "",
            ".",
            "..",
            "../etc/passwd",
            "foo/../bar",
            "/absolute",
            "\\windows",
            "a/b",
            "a\\b",
            "foo..bar", // defence-in-depth: still contains ".."
            "with space",
            "has\0null",
            "new\nline",
            "emoji🚀",
        ] {
            let res = validate_proposal_id(bad);
            assert!(res.is_err(), "expected {bad:?} to be rejected");
            assert_eq!(res.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn create_self_mod_worktree_rejects_hostile_proposal_id() {
        let tmp = tempfile::tempdir().expect("failed to create tempdir");
        let err = create_self_mod_worktree(tmp.path(), "../escape")
            .expect_err("hostile proposal_id must be rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        // No directory named "escape" should have been created anywhere
        // outside the worktrees root.
        assert!(!tmp.path().join("escape").exists());
    }

    // -----------------------------------------------------------------------
    // classify_modification_target — tightened segment semantics
    // -----------------------------------------------------------------------

    #[test]
    fn classify_does_not_misfire_on_substring_collisions() {
        // `promptly-archive` happens to contain "prompt" but must not be
        // classified as a Prompt target.
        assert_eq!(
            classify_modification_target("promptly-archive/foo.rs"),
            ModificationTarget::Other,
        );
        // A directory named `configuration` (without `config` segment match)
        // should not misclassify an arbitrary `.toml`.
        assert_eq!(
            classify_modification_target("configuration-notes/readme.toml"),
            ModificationTarget::Other,
        );
        // A `.rs` file *not* under `crates/` should not classify as Source.
        assert_eq!(
            classify_modification_target("fuzz/fuzz_targets/foo.rs"),
            ModificationTarget::Other,
        );
    }

    #[test]
    fn classify_template_filename_still_classifies_as_prompt() {
        // Preserves the original contract: `assets/template.hbs` → Prompt
        // even without a dedicated `templates/` segment, because the
        // filename stem starts with `template`.
        assert_eq!(
            classify_modification_target("assets/template.hbs"),
            ModificationTarget::Prompt,
        );
    }

    #[test]
    fn classify_windows_style_paths_are_normalised() {
        // Windows-style separators classify the same as POSIX.
        assert_eq!(
            classify_modification_target(r"crates\tau-ops\src\main.rs"),
            ModificationTarget::Source,
        );
        assert_eq!(
            classify_modification_target(r"skills\my-skill\manifest.toml"),
            ModificationTarget::Skill,
        );
    }
}
