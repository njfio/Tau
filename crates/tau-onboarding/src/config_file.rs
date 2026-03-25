//! Declarative `.tau.toml` configuration parser and generator.
//!
//! Provides the `TauConfig` struct that mirrors the `.tau.toml` schema,
//! along with `load_tau_config` for reading from disk and
//! `generate_default_config` for producing a commented default file.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when loading or parsing a `.tau.toml` config file.
#[derive(Debug)]
pub enum ConfigError {
    /// The file could not be read from disk.
    Io(std::io::Error),
    /// The file contents are not valid TOML or do not match the schema.
    Parse(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "config I/O error: {e}"),
            ConfigError::Parse(e) => write!(f, "config parse error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Parse(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}

// ---------------------------------------------------------------------------
// Sub-config structs
// ---------------------------------------------------------------------------

/// Agent identity and model configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentConfig {
    #[serde(default = "default_agent_name")]
    pub name: String,
    #[serde(default = "default_agent_model")]
    pub model: String,
    #[serde(default)]
    pub fallback_models: Vec<String>,
}

fn default_agent_name() -> String {
    "tau-agent".to_string()
}

fn default_agent_model() -> String {
    "claude-sonnet-4-6".to_string()
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: default_agent_name(),
            model: default_agent_model(),
            fallback_models: vec!["claude-haiku-4-5-20251001".to_string()],
        }
    }
}

/// Session persistence configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_session_path")]
    pub path: String,
    #[serde(default = "default_session_import_mode")]
    pub import_mode: String,
}

fn default_true() -> bool {
    true
}

fn default_session_path() -> String {
    ".tau/sessions".to_string()
}

fn default_session_import_mode() -> String {
    "auto".to_string()
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: default_session_path(),
            import_mode: default_session_import_mode(),
        }
    }
}

/// Tool and sandbox policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyConfig {
    #[serde(default = "default_tool_policy_preset")]
    pub tool_policy_preset: String,
    #[serde(default = "default_bash_profile")]
    pub bash_profile: String,
    #[serde(default = "default_os_sandbox_mode")]
    pub os_sandbox_mode: String,
    #[serde(default = "default_bash_timeout_ms")]
    pub bash_timeout_ms: u64,
}

fn default_tool_policy_preset() -> String {
    "standard".to_string()
}

fn default_bash_profile() -> String {
    "default".to_string()
}

fn default_os_sandbox_mode() -> String {
    "relaxed".to_string()
}

fn default_bash_timeout_ms() -> u64 {
    30_000
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            tool_policy_preset: default_tool_policy_preset(),
            bash_profile: default_bash_profile(),
            os_sandbox_mode: default_os_sandbox_mode(),
            bash_timeout_ms: default_bash_timeout_ms(),
        }
    }
}

/// Memory and learning configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryConfig {
    #[serde(default = "default_true")]
    pub action_history_enabled: bool,
    #[serde(default = "default_retention_days")]
    pub action_history_retention_days: u32,
    #[serde(default = "default_true")]
    pub cortex_enabled: bool,
}

fn default_retention_days() -> u32 {
    30
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            action_history_enabled: true,
            action_history_retention_days: default_retention_days(),
            cortex_enabled: true,
        }
    }
}

/// Training pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrainingConfig {
    #[serde(default = "default_true")]
    pub live_rl_enabled: bool,
    #[serde(default = "default_apo_threshold")]
    pub apo_auto_trigger_threshold: usize,
}

fn default_apo_threshold() -> usize {
    20
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            live_rl_enabled: true,
            apo_auto_trigger_threshold: default_apo_threshold(),
        }
    }
}

/// Safety policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SafetyConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_safety_mode")]
    pub mode: String,
    #[serde(default = "default_true")]
    pub secret_leak_detection: bool,
}

fn default_safety_mode() -> String {
    "warn".to_string()
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: default_safety_mode(),
            secret_leak_detection: true,
        }
    }
}

/// Per-channel enablement toggle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChannelToggle {
    #[serde(default)]
    pub enabled: bool,
}

/// Multi-channel runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ChannelsConfig {
    #[serde(default)]
    pub slack: ChannelToggle,
    #[serde(default)]
    pub discord: ChannelToggle,
    #[serde(default)]
    pub github_issues: ChannelToggle,
}

/// Skills loading and auto-selection configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillsConfig {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default = "default_true")]
    pub auto_select: bool,
}

impl Default for SkillsConfig {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            auto_select: true,
        }
    }
}

/// Provider authentication mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthConfig {
    #[serde(default = "default_auth_mode")]
    pub openai_auth_mode: String,
    #[serde(default = "default_auth_mode")]
    pub anthropic_auth_mode: String,
}

fn default_auth_mode() -> String {
    "api_key".to_string()
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            openai_auth_mode: default_auth_mode(),
            anthropic_auth_mode: default_auth_mode(),
        }
    }
}

/// Self-improvement / reflexive patching configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelfImprovementConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_apply_skills: bool,
    #[serde(default)]
    pub auto_apply_config: bool,
    #[serde(default)]
    pub auto_apply_source: bool,
}

impl Default for SelfImprovementConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_apply_skills: false,
            auto_apply_config: false,
            auto_apply_source: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// Root configuration struct mirroring the `.tau.toml` schema.
///
/// Every section is optional and falls back to sensible defaults when absent.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TauConfig {
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub session: SessionConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub training: TrainingConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub channels: ChannelsConfig,
    #[serde(default)]
    pub skills: SkillsConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub self_improvement: SelfImprovementConfig,
}

// ---------------------------------------------------------------------------
// Conversion to ProfileDefaults
// ---------------------------------------------------------------------------

impl TauConfig {
    /// Convert this `TauConfig` into `ProfileDefaults`, mapping each config
    /// section to the corresponding profile fields and using sensible defaults
    /// for any fields that `.tau.toml` does not cover.
    pub fn to_profile_defaults(&self) -> crate::startup_config::ProfileDefaults {
        use crate::startup_config::*;

        let auth_method_from_str = |s: &str| -> tau_provider::ProviderAuthMethod {
            match s {
                "oauth" | "oauth_token" => tau_provider::ProviderAuthMethod::OauthToken,
                "adc" => tau_provider::ProviderAuthMethod::Adc,
                "session_token" => tau_provider::ProviderAuthMethod::SessionToken,
                _ => tau_provider::ProviderAuthMethod::ApiKey,
            }
        };

        ProfileDefaults {
            model: self.agent.model.clone(),
            fallback_models: self.agent.fallback_models.clone(),
            session: ProfileSessionDefaults {
                enabled: self.session.enabled,
                path: if self.session.enabled {
                    Some(self.session.path.clone())
                } else {
                    None
                },
                import_mode: self.session.import_mode.clone(),
            },
            policy: ProfilePolicyDefaults {
                tool_policy_preset: self.policy.tool_policy_preset.clone(),
                bash_profile: self.policy.bash_profile.clone(),
                bash_dry_run: false,
                os_sandbox_mode: self.policy.os_sandbox_mode.clone(),
                enforce_regular_files: false,
                bash_timeout_ms: self.policy.bash_timeout_ms,
                max_command_length: 4_096,
                max_tool_output_bytes: 16_000,
                max_file_read_bytes: 1_000_000,
                max_file_write_bytes: 1_000_000,
                allow_command_newlines: false,
                runtime_heartbeat_enabled: true,
                runtime_heartbeat_interval_ms: 5_000,
                runtime_heartbeat_state_path: ".tau/runtime-heartbeat/state.json".to_string(),
                runtime_self_repair_enabled: true,
                runtime_self_repair_timeout_ms: 300_000,
                runtime_self_repair_max_retries: 2,
                runtime_self_repair_tool_builds_dir: ".tau/tool-builds".to_string(),
                runtime_self_repair_orphan_max_age_seconds: 3_600,
                context_compaction_warn_threshold_percent: 80,
                context_compaction_aggressive_threshold_percent: 85,
                context_compaction_emergency_threshold_percent: 95,
                context_compaction_warn_retain_percent: 70,
                context_compaction_aggressive_retain_percent: 50,
                context_compaction_emergency_retain_percent: 50,
            },
            mcp: ProfileMcpDefaults::default(),
            auth: ProfileAuthDefaults {
                openai: auth_method_from_str(&self.auth.openai_auth_mode),
                anthropic: auth_method_from_str(&self.auth.anthropic_auth_mode),
                google: tau_provider::ProviderAuthMethod::ApiKey,
            },
            routing: ProfileRoutingDefaults::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load and parse a `.tau.toml` configuration file from disk.
///
/// Returns `ConfigError::Io` if the file cannot be read, or
/// `ConfigError::Parse` if the contents are not valid TOML or do not
/// conform to the `TauConfig` schema.
pub fn load_tau_config(path: &Path) -> Result<TauConfig, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    let config: TauConfig = toml::from_str(&contents)?;
    Ok(config)
}

/// Parse a `.tau.toml` string directly into a `TauConfig`.
///
/// Useful for testing and for scenarios where the TOML content is already
/// available in memory.
pub fn parse_tau_config(toml_str: &str) -> Result<TauConfig, ConfigError> {
    let config: TauConfig = toml::from_str(toml_str)?;
    Ok(config)
}

/// Generate a default `.tau.toml` file with all sections and comments.
///
/// The output is intended for `tau init --auto` and includes inline
/// documentation for every field.
pub fn generate_default_config() -> String {
    r#"# Tau Agent Configuration
# Generated by `tau init --auto`
# Documentation: https://tau.dev/docs/configuration

[agent]
name = "tau-agent"
model = "claude-sonnet-4-6"
fallback_models = ["claude-haiku-4-5-20251001"]

[session]
enabled = true
path = ".tau/sessions"
import_mode = "auto"

[policy]
tool_policy_preset = "standard"
bash_profile = "default"
os_sandbox_mode = "relaxed"
bash_timeout_ms = 30000

[memory]
action_history_enabled = true
action_history_retention_days = 30
cortex_enabled = true

[training]
live_rl_enabled = true
apo_auto_trigger_threshold = 20

[safety]
enabled = true
mode = "warn"
secret_leak_detection = true

[channels]
# Enable additional channels as needed.

[channels.slack]
enabled = false

[channels.discord]
enabled = false

[channels.github_issues]
enabled = false

[skills]
include = []
auto_select = true

[auth]
openai_auth_mode = "api_key"
anthropic_auth_mode = "api_key"

[self_improvement]
enabled = false
auto_apply_skills = false
auto_apply_config = false
auto_apply_source = false
"#
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Parse a fully-specified valid TOML string and verify values.
    #[test]
    fn test_parse_valid_tau_toml() {
        let toml_str = r#"
[agent]
name = "my-custom-agent"
model = "openai/gpt-5.2"
fallback_models = ["openai/gpt-4.1-mini"]

[session]
enabled = false
path = "/tmp/sessions"
import_mode = "merge"

[policy]
tool_policy_preset = "balanced"
bash_profile = "balanced"
os_sandbox_mode = "strict"
bash_timeout_ms = 60000

[memory]
action_history_enabled = false
action_history_retention_days = 7
cortex_enabled = false

[training]
live_rl_enabled = false
apo_auto_trigger_threshold = 50

[safety]
enabled = false
mode = "block"
secret_leak_detection = false

[channels.slack]
enabled = true

[channels.discord]
enabled = true

[channels.github_issues]
enabled = true

[skills]
include = ["web-game-phaser", "data-analysis"]
auto_select = false

[auth]
openai_auth_mode = "oauth"
anthropic_auth_mode = "oauth"

[self_improvement]
enabled = true
auto_apply_skills = true
auto_apply_config = true
auto_apply_source = false
"#;
        let config = parse_tau_config(toml_str).expect("valid TOML should parse");

        assert_eq!(config.agent.name, "my-custom-agent");
        assert_eq!(config.agent.model, "openai/gpt-5.2");
        assert_eq!(config.agent.fallback_models, vec!["openai/gpt-4.1-mini"]);

        assert!(!config.session.enabled);
        assert_eq!(config.session.path, "/tmp/sessions");
        assert_eq!(config.session.import_mode, "merge");

        assert_eq!(config.policy.tool_policy_preset, "balanced");
        assert_eq!(config.policy.bash_profile, "balanced");
        assert_eq!(config.policy.os_sandbox_mode, "strict");
        assert_eq!(config.policy.bash_timeout_ms, 60_000);

        assert!(!config.memory.action_history_enabled);
        assert_eq!(config.memory.action_history_retention_days, 7);
        assert!(!config.memory.cortex_enabled);

        assert!(!config.training.live_rl_enabled);
        assert_eq!(config.training.apo_auto_trigger_threshold, 50);

        assert!(!config.safety.enabled);
        assert_eq!(config.safety.mode, "block");
        assert!(!config.safety.secret_leak_detection);

        assert!(config.channels.slack.enabled);
        assert!(config.channels.discord.enabled);
        assert!(config.channels.github_issues.enabled);

        assert_eq!(
            config.skills.include,
            vec!["web-game-phaser", "data-analysis"]
        );
        assert!(!config.skills.auto_select);

        assert_eq!(config.auth.openai_auth_mode, "oauth");
        assert_eq!(config.auth.anthropic_auth_mode, "oauth");

        assert!(config.self_improvement.enabled);
        assert!(config.self_improvement.auto_apply_skills);
        assert!(config.self_improvement.auto_apply_config);
        assert!(!config.self_improvement.auto_apply_source);
    }

    /// Test 2: Empty TOML produces TauConfig with all defaults.
    #[test]
    fn test_parse_empty_toml_returns_all_defaults() {
        let config = parse_tau_config("").expect("empty TOML should parse");
        let defaults = TauConfig::default();
        assert_eq!(config, defaults);

        // Spot-check specific defaults from the plan
        assert_eq!(config.agent.name, "tau-agent");
        assert_eq!(config.agent.model, "claude-sonnet-4-6");
        assert!(config.session.enabled);
        assert_eq!(config.policy.bash_timeout_ms, 30_000);
        assert!(config.memory.action_history_enabled);
        assert_eq!(config.memory.action_history_retention_days, 30);
        assert!(config.training.live_rl_enabled);
        assert_eq!(config.training.apo_auto_trigger_threshold, 20);
        assert!(config.safety.enabled);
        assert_eq!(config.safety.mode, "warn");
        assert!(!config.channels.slack.enabled);
        assert!(config.skills.auto_select);
        assert!(!config.self_improvement.enabled);
    }

    /// Test 3: Partial TOML (only [agent]) fills rest with defaults.
    #[test]
    fn test_parse_partial_toml_agent_only() {
        let toml_str = r#"
[agent]
name = "partial-agent"
model = "anthropic/claude-opus-4"
"#;
        let config = parse_tau_config(toml_str).expect("partial TOML should parse");

        assert_eq!(config.agent.name, "partial-agent");
        assert_eq!(config.agent.model, "anthropic/claude-opus-4");
        // fallback_models defaults to empty vec (from serde default)
        assert!(config.agent.fallback_models.is_empty());

        // All other sections should be at defaults
        assert_eq!(config.session, SessionConfig::default());
        assert_eq!(config.policy, PolicyConfig::default());
        assert_eq!(config.memory, MemoryConfig::default());
        assert_eq!(config.training, TrainingConfig::default());
        assert_eq!(config.safety, SafetyConfig::default());
        assert_eq!(config.channels, ChannelsConfig::default());
        assert_eq!(config.skills, SkillsConfig::default());
        assert_eq!(config.auth, AuthConfig::default());
        assert_eq!(config.self_improvement, SelfImprovementConfig::default());
    }

    /// Test 4: generate_default_config() produces TOML that can be parsed back.
    #[test]
    fn test_generate_default_config_is_parseable() {
        let generated = generate_default_config();
        let config =
            parse_tau_config(&generated).expect("generated default config should be parseable");

        assert_eq!(config.agent.name, "tau-agent");
        assert_eq!(config.agent.model, "claude-sonnet-4-6");
        assert_eq!(
            config.agent.fallback_models,
            vec!["claude-haiku-4-5-20251001"]
        );
        assert!(config.session.enabled);
        assert_eq!(config.session.path, ".tau/sessions");
        assert_eq!(config.policy.tool_policy_preset, "standard");
        assert_eq!(config.policy.bash_timeout_ms, 30_000);
        assert!(config.memory.action_history_enabled);
        assert_eq!(config.memory.action_history_retention_days, 30);
        assert!(config.training.live_rl_enabled);
        assert_eq!(config.training.apo_auto_trigger_threshold, 20);
        assert!(config.safety.enabled);
        assert_eq!(config.safety.mode, "warn");
        assert!(config.safety.secret_leak_detection);
        assert!(!config.channels.slack.enabled);
        assert!(!config.channels.discord.enabled);
        assert!(!config.channels.github_issues.enabled);
        assert!(config.skills.include.is_empty());
        assert!(config.skills.auto_select);
        assert_eq!(config.auth.openai_auth_mode, "api_key");
        assert_eq!(config.auth.anthropic_auth_mode, "api_key");
        assert!(!config.self_improvement.enabled);
    }

    /// Test 5: Invalid TOML returns ConfigError::Parse.
    #[test]
    fn test_invalid_toml_returns_parse_error() {
        let bad_toml = r#"
[agent
name = "broken
"#;
        let result = parse_tau_config(bad_toml);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::Parse(_) => {} // expected
            other => panic!("expected ConfigError::Parse, got: {other}"),
        }
    }

    /// Test: load_tau_config returns Io error for non-existent file.
    #[test]
    fn test_load_nonexistent_file_returns_io_error() {
        let result = load_tau_config(Path::new("/tmp/does-not-exist-tau-config-test.toml"));
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::Io(_) => {} // expected
            other => panic!("expected ConfigError::Io, got: {other}"),
        }
    }

    /// Test: load_tau_config works with a real file on disk.
    #[test]
    fn test_load_tau_config_from_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join(".tau.toml");
        std::fs::write(
            &path,
            r#"
[agent]
name = "file-test"
model = "test-model"
"#,
        )
        .expect("write temp file");

        let config = load_tau_config(&path).expect("should parse file");
        assert_eq!(config.agent.name, "file-test");
        assert_eq!(config.agent.model, "test-model");
        // Rest should be defaults
        assert!(config.session.enabled);
    }

    /// Test: to_profile_defaults maps default TauConfig correctly.
    #[test]
    fn test_to_profile_defaults_from_default_config() {
        let config = TauConfig::default();
        let defaults = config.to_profile_defaults();

        assert_eq!(defaults.model, "claude-sonnet-4-6");
        assert_eq!(
            defaults.fallback_models,
            vec!["claude-haiku-4-5-20251001".to_string()]
        );
        assert!(defaults.session.enabled);
        assert_eq!(defaults.session.path, Some(".tau/sessions".to_string()));
        assert_eq!(defaults.session.import_mode, "auto");
        assert_eq!(defaults.policy.tool_policy_preset, "standard");
        assert_eq!(defaults.policy.bash_profile, "default");
        assert_eq!(defaults.policy.os_sandbox_mode, "relaxed");
        assert_eq!(defaults.policy.bash_timeout_ms, 30_000);
        assert_eq!(
            defaults.auth.openai,
            tau_provider::ProviderAuthMethod::ApiKey
        );
        assert_eq!(
            defaults.auth.anthropic,
            tau_provider::ProviderAuthMethod::ApiKey
        );
    }

    /// Test: to_profile_defaults maps custom TauConfig values.
    #[test]
    fn test_to_profile_defaults_from_custom_config() {
        let config = parse_tau_config(
            r#"
[agent]
model = "openai/gpt-5.2"
fallback_models = ["openai/gpt-4.1-mini"]

[session]
enabled = false
path = "/tmp/sessions"
import_mode = "merge"

[policy]
tool_policy_preset = "balanced"
bash_profile = "strict"
os_sandbox_mode = "strict"
bash_timeout_ms = 60000

[auth]
openai_auth_mode = "oauth"
anthropic_auth_mode = "session_token"
"#,
        )
        .expect("parse");
        let defaults = config.to_profile_defaults();

        assert_eq!(defaults.model, "openai/gpt-5.2");
        assert_eq!(defaults.fallback_models, vec!["openai/gpt-4.1-mini"]);
        assert!(!defaults.session.enabled);
        assert_eq!(defaults.session.path, None);
        assert_eq!(defaults.session.import_mode, "merge");
        assert_eq!(defaults.policy.tool_policy_preset, "balanced");
        assert_eq!(defaults.policy.bash_profile, "strict");
        assert_eq!(defaults.policy.os_sandbox_mode, "strict");
        assert_eq!(defaults.policy.bash_timeout_ms, 60_000);
        assert_eq!(
            defaults.auth.openai,
            tau_provider::ProviderAuthMethod::OauthToken
        );
        assert_eq!(
            defaults.auth.anthropic,
            tau_provider::ProviderAuthMethod::SessionToken
        );
    }

    /// Test: ConfigError Display formatting.
    #[test]
    fn test_config_error_display() {
        let io_err = ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        let display = format!("{io_err}");
        assert!(display.contains("config I/O error"));

        let parse_err = parse_tau_config("[agent\n").unwrap_err();
        let display = format!("{parse_err}");
        assert!(display.contains("config parse error"));
    }
}
