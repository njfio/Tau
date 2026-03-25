//! Self-modification runtime with worktree isolation.
//!
//! Provides configuration, classification, and worktree management for
//! autonomous source-code self-modification proposals. Safety evaluation
//! results are represented locally so this module can be used without
//! pulling in the full `tau-safety` crate at the type level.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
pub fn classify_modification_target(path: &str) -> ModificationTarget {
    if path.starts_with("skills/") || (path.ends_with(".md") && path.contains("skills")) {
        ModificationTarget::Skill
    } else if path == ".tau.toml" || (path.contains("config") && path.ends_with(".toml")) {
        ModificationTarget::Config
    } else if path.contains("prompt") || path.contains("template") {
        ModificationTarget::Prompt
    } else if path.contains("crates/") && path.ends_with(".rs") {
        ModificationTarget::Source
    } else {
        ModificationTarget::Other
    }
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

/// Create a worktree directory for isolated source modification.
///
/// In production this would shell out to `git worktree add`; the current
/// implementation creates the directory structure so that later stages can
/// populate it.
pub fn create_self_mod_worktree(
    workspace_root: &Path,
    proposal_id: &str,
) -> Result<PathBuf, std::io::Error> {
    let worktree_path = workspace_root
        .join(".tau")
        .join("self-mod-worktrees")
        .join(proposal_id);
    std::fs::create_dir_all(&worktree_path)?;
    Ok(worktree_path)
}

/// Remove a self-modification worktree that is no longer needed.
pub fn cleanup_self_mod_worktree(worktree_path: &Path) -> Result<(), std::io::Error> {
    if worktree_path.exists() {
        std::fs::remove_dir_all(worktree_path)?;
    }
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
        let worktree_path = create_self_mod_worktree(tmp.path(), "test-cleanup-1")
            .expect("create should succeed");
        assert!(worktree_path.exists());

        let result = cleanup_self_mod_worktree(&worktree_path);
        assert!(result.is_ok());
        assert!(!worktree_path.exists());
    }

    #[test]
    fn cleanup_self_mod_worktree_succeeds_when_path_does_not_exist() {
        let nonexistent = PathBuf::from("/tmp/tau-test-nonexistent-worktree-path");
        assert!(!nonexistent.exists());
        let result = cleanup_self_mod_worktree(&nonexistent);
        assert!(result.is_ok());
    }
}
