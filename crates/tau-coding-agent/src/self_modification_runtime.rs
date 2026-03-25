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

// ---------------------------------------------------------------------------
// B2: Skill effectiveness check — triggers self-modification when a skill
// underperforms across a sufficient number of tracked sessions.
// ---------------------------------------------------------------------------

/// Result of evaluating whether a skill's effectiveness warrants
/// self-modification (e.g. rewriting skill prompts or parameters).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEffectivenessCheck {
    pub skill_name: String,
    pub success_rate: f64,
    pub sessions_tracked: usize,
    pub threshold: f64,
    pub should_trigger: bool,
}

/// Evaluate whether a skill's success rate is below the acceptable threshold
/// after a minimum number of tracked sessions.
///
/// Returns a [`SkillEffectivenessCheck`] with `should_trigger = true` when
/// `sessions_tracked >= min_sessions` **and** `success_rate < threshold`.
pub fn check_skill_effectiveness(
    skill_name: &str,
    success_rate: f64,
    sessions_tracked: usize,
    min_sessions: usize,
    threshold: f64,
) -> SkillEffectivenessCheck {
    SkillEffectivenessCheck {
        skill_name: skill_name.to_string(),
        success_rate,
        sessions_tracked,
        threshold,
        should_trigger: sessions_tracked >= min_sessions && success_rate < threshold,
    }
}

// ---------------------------------------------------------------------------
// B4: Source code self-modification pipeline with worktree isolation.
// ---------------------------------------------------------------------------

/// Stage of a source-code self-modification pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceModificationStage {
    Proposed,
    WorktreeCreated,
    PatchApplied,
    TestsRunning,
    TestsPassed,
    TestsFailed,
    SafetyReview,
    SafetyCleared,
    SafetyBlocked,
    PrCreated,
    HumanApproval,
    Merged,
    RolledBack,
}

/// A pipeline tracking the lifecycle of a source-code self-modification from
/// proposal through worktree creation, testing, safety review, and merge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceModificationPipeline {
    /// The target file path (relative to repo root).
    pub target_path: String,
    /// The unified diff describing the proposed change.
    pub diff: String,
    /// Human-readable rationale for the modification.
    pub rationale: String,
    /// Path to the git worktree used for isolated testing.
    pub worktree_path: Option<PathBuf>,
    /// Whether the test suite passed in the worktree.
    pub test_passed: Option<bool>,
    /// Whether the safety review cleared the change.
    pub safety_cleared: Option<bool>,
    /// URL of the pull request created for human review.
    pub pr_url: Option<String>,
    /// Current stage of the pipeline.
    pub stage: SourceModificationStage,
}

impl SourceModificationPipeline {
    /// Create a new pipeline in the `Proposed` stage.
    pub fn new(target_path: String, diff: String, rationale: String) -> Self {
        Self {
            target_path,
            diff,
            rationale,
            worktree_path: None,
            test_passed: None,
            safety_cleared: None,
            pr_url: None,
            stage: SourceModificationStage::Proposed,
        }
    }

    /// Advance the pipeline to the given stage.
    pub fn advance_to(&mut self, stage: SourceModificationStage) {
        self.stage = stage;
    }

    /// Returns `true` when the pipeline has reached a terminal state from
    /// which no further transitions are expected.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.stage,
            SourceModificationStage::Merged
                | SourceModificationStage::RolledBack
                | SourceModificationStage::SafetyBlocked
                | SourceModificationStage::TestsFailed
        )
    }
}

/// Remove a self-modification worktree that is no longer needed.
pub fn cleanup_self_mod_worktree(worktree_path: &Path) -> Result<(), std::io::Error> {
    if worktree_path.exists() {
        std::fs::remove_dir_all(worktree_path)?;
    }
    Ok(())
}

/// B8: Reward signal for a self-modification proposal.
///
/// Tracks before/after effectiveness to determine whether a self-modification
/// improved or regressed system behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationReward {
    pub proposal_id: String,
    pub sessions_before: usize,
    pub sessions_after: usize,
    pub effectiveness_before: f64,
    pub effectiveness_after: f64,
    /// Positive = improvement, negative = regression.
    pub reward: f64,
    /// True if enough sessions have elapsed and the reward is significantly negative.
    pub should_rollback: bool,
}

/// B8: Compute a reward signal for a self-modification proposal.
///
/// Compares effectiveness before and after the modification was applied.
/// Recommends rollback when enough sessions (`min_sessions`) have been
/// observed and the effectiveness dropped by more than 0.1.
pub fn compute_self_modification_reward(
    proposal_id: &str,
    effectiveness_before: f64,
    effectiveness_after: f64,
    min_sessions: usize,
    sessions_after: usize,
) -> SelfModificationReward {
    let reward = effectiveness_after - effectiveness_before;
    SelfModificationReward {
        proposal_id: proposal_id.to_string(),
        sessions_before: min_sessions,
        sessions_after,
        effectiveness_before,
        effectiveness_after,
        reward,
        should_rollback: sessions_after >= min_sessions && reward < -0.1,
    }
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

    // -------------------------------------------------------------------
    // B2: check_skill_effectiveness
    // -------------------------------------------------------------------

    #[test]
    fn skill_effectiveness_triggers_when_below_threshold_and_enough_sessions() {
        let check = check_skill_effectiveness("code-review", 0.4, 10, 5, 0.7);
        assert!(check.should_trigger);
        assert_eq!(check.skill_name, "code-review");
        assert!((check.success_rate - 0.4).abs() < f64::EPSILON);
        assert_eq!(check.sessions_tracked, 10);
        assert!((check.threshold - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn skill_effectiveness_does_not_trigger_when_above_threshold() {
        let check = check_skill_effectiveness("code-review", 0.85, 10, 5, 0.7);
        assert!(!check.should_trigger);
    }

    #[test]
    fn skill_effectiveness_does_not_trigger_when_insufficient_sessions() {
        let check = check_skill_effectiveness("code-review", 0.3, 3, 5, 0.7);
        assert!(!check.should_trigger);
    }

    #[test]
    fn skill_effectiveness_triggers_at_exact_boundary() {
        // Exactly at min_sessions, and rate < threshold
        let check = check_skill_effectiveness("deploy", 0.69, 5, 5, 0.7);
        assert!(check.should_trigger);
    }

    #[test]
    fn skill_effectiveness_does_not_trigger_at_exact_threshold_rate() {
        // rate == threshold is NOT below threshold
        let check = check_skill_effectiveness("deploy", 0.7, 10, 5, 0.7);
        assert!(!check.should_trigger);
    }

    // -------------------------------------------------------------------
    // B4: SourceModificationPipeline
    // -------------------------------------------------------------------

    #[test]
    fn pipeline_starts_in_proposed_stage() {
        let pipeline = SourceModificationPipeline::new(
            "crates/tau-ops/src/lib.rs".to_string(),
            "--- a\n+++ b\n".to_string(),
            "improve performance".to_string(),
        );
        assert_eq!(pipeline.stage, SourceModificationStage::Proposed);
        assert!(!pipeline.is_terminal());
    }

    #[test]
    fn pipeline_advance_to_changes_stage() {
        let mut pipeline = SourceModificationPipeline::new(
            "src/main.rs".to_string(),
            "diff".to_string(),
            "fix bug".to_string(),
        );
        pipeline.advance_to(SourceModificationStage::WorktreeCreated);
        assert_eq!(pipeline.stage, SourceModificationStage::WorktreeCreated);
        assert!(!pipeline.is_terminal());
    }

    #[test]
    fn pipeline_merged_is_terminal() {
        let mut pipeline = SourceModificationPipeline::new(
            "src/main.rs".to_string(),
            "diff".to_string(),
            "reason".to_string(),
        );
        pipeline.advance_to(SourceModificationStage::Merged);
        assert!(pipeline.is_terminal());
    }

    #[test]
    fn pipeline_rolled_back_is_terminal() {
        let mut pipeline = SourceModificationPipeline::new(
            "src/main.rs".to_string(),
            "diff".to_string(),
            "reason".to_string(),
        );
        pipeline.advance_to(SourceModificationStage::RolledBack);
        assert!(pipeline.is_terminal());
    }

    #[test]
    fn pipeline_safety_blocked_is_terminal() {
        let mut pipeline = SourceModificationPipeline::new(
            "src/main.rs".to_string(),
            "diff".to_string(),
            "reason".to_string(),
        );
        pipeline.advance_to(SourceModificationStage::SafetyBlocked);
        assert!(pipeline.is_terminal());
    }

    #[test]
    fn pipeline_tests_failed_is_terminal() {
        let mut pipeline = SourceModificationPipeline::new(
            "src/main.rs".to_string(),
            "diff".to_string(),
            "reason".to_string(),
        );
        pipeline.advance_to(SourceModificationStage::TestsFailed);
        assert!(pipeline.is_terminal());
    }

    #[test]
    fn pipeline_non_terminal_stages() {
        let non_terminal = vec![
            SourceModificationStage::Proposed,
            SourceModificationStage::WorktreeCreated,
            SourceModificationStage::PatchApplied,
            SourceModificationStage::TestsRunning,
            SourceModificationStage::TestsPassed,
            SourceModificationStage::SafetyReview,
            SourceModificationStage::SafetyCleared,
            SourceModificationStage::PrCreated,
            SourceModificationStage::HumanApproval,
        ];
        for stage in non_terminal {
            let mut pipeline = SourceModificationPipeline::new(
                "f.rs".to_string(),
                "d".to_string(),
                "r".to_string(),
            );
            pipeline.advance_to(stage.clone());
            assert!(
                !pipeline.is_terminal(),
                "stage {:?} should not be terminal",
                stage
            );
        }
    }

    #[test]
    fn pipeline_round_trips_through_json() {
        let pipeline = SourceModificationPipeline::new(
            "crates/tau-ops/src/lib.rs".to_string(),
            "--- a\n+++ b\n".to_string(),
            "testing serde".to_string(),
        );
        let json = serde_json::to_string(&pipeline).expect("serialize");
        let deserialized: SourceModificationPipeline =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.target_path, pipeline.target_path);
        assert_eq!(deserialized.stage, SourceModificationStage::Proposed);
    }

    // -------------------------------------------------------------------
    // B8: compute_self_modification_reward
    // -------------------------------------------------------------------

    #[test]
    fn reward_positive_when_effectiveness_improves() {
        let reward = compute_self_modification_reward("p-001", 0.6, 0.8, 5, 10);
        assert!((reward.reward - 0.2).abs() < f64::EPSILON);
        assert!(!reward.should_rollback);
        assert_eq!(reward.proposal_id, "p-001");
    }

    #[test]
    fn reward_negative_when_effectiveness_regresses() {
        let reward = compute_self_modification_reward("p-002", 0.8, 0.5, 5, 10);
        assert!((reward.reward - (-0.3)).abs() < f64::EPSILON);
        assert!(reward.should_rollback);
    }

    #[test]
    fn reward_no_rollback_when_insufficient_sessions() {
        // Even with negative reward, don't rollback if not enough sessions observed
        let reward = compute_self_modification_reward("p-003", 0.8, 0.5, 10, 5);
        assert!(reward.reward < -0.1);
        assert!(!reward.should_rollback); // sessions_after < min_sessions
    }

    #[test]
    fn reward_no_rollback_when_regression_is_small() {
        // Small regression (< 0.1) should not trigger rollback
        let reward = compute_self_modification_reward("p-004", 0.8, 0.75, 5, 10);
        assert!((reward.reward - (-0.05)).abs() < f64::EPSILON);
        assert!(!reward.should_rollback);
    }

    #[test]
    fn reward_serializes_correctly() {
        let reward = compute_self_modification_reward("p-005", 0.7, 0.9, 5, 10);
        let json = serde_json::to_string(&reward).expect("serialize");
        let deser: SelfModificationReward = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.proposal_id, "p-005");
        assert!((deser.reward - 0.2).abs() < f64::EPSILON);
    }
}
