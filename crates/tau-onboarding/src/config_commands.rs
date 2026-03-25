//! Config command handlers for `/config-validate`, `/config-show`, and `/init`.
//!
//! These handlers operate on `.tau.toml` configuration files and return
//! user-facing strings suitable for TUI/CLI display.

use std::path::Path;

use crate::config_file::{generate_default_config, load_tau_config};

/// Validate a `.tau.toml` configuration file at the given path.
///
/// Returns a human-readable string indicating success or the specific error.
pub fn handle_config_validate(path: &Path) -> String {
    match load_tau_config(path) {
        Ok(_) => "Configuration valid.".to_string(),
        Err(e) => format!("Configuration error: {e}"),
    }
}

/// Show the resolved configuration from a `.tau.toml` file, annotating each
/// value with its source: "from .tau.toml", "from env", or "default".
///
/// The `env_overrides` parameter is a list of `(key, value)` pairs representing
/// environment variable overrides that were applied on top of the file config.
pub fn handle_config_show(path: &Path, env_overrides: &[(String, String)]) -> String {
    let config = match load_tau_config(path) {
        Ok(c) => c,
        Err(e) => return format!("Failed to load config: {e}"),
    };
    let defaults = crate::config_file::TauConfig::default();

    let env_map: std::collections::HashMap<&str, &str> = env_overrides
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let mut lines = Vec::new();
    lines.push("# Resolved Tau Configuration".to_string());
    lines.push(String::new());

    // Helper macro to annotate a field
    macro_rules! show_field {
        ($section:expr, $field:expr, $val:expr, $default_val:expr, $env_key:expr) => {
            let source = if let Some(env_val) = env_map.get($env_key) {
                format!("{} = \"{}\"  # from env ({})", $field, env_val, $env_key)
            } else if $val != $default_val {
                format!("{} = {:?}  # from .tau.toml", $field, $val)
            } else {
                format!("{} = {:?}  # default", $field, $val)
            };
            lines.push(source);
        };
    }

    lines.push("[agent]".to_string());
    show_field!("agent", "name", config.agent.name, defaults.agent.name, "TAU_AGENT_NAME");
    show_field!("agent", "model", config.agent.model, defaults.agent.model, "TAU_MODEL");
    show_field!(
        "agent",
        "fallback_models",
        config.agent.fallback_models,
        defaults.agent.fallback_models,
        "TAU_FALLBACK_MODELS"
    );

    lines.push(String::new());
    lines.push("[session]".to_string());
    show_field!("session", "enabled", config.session.enabled, defaults.session.enabled, "TAU_SESSION_ENABLED");
    show_field!("session", "path", config.session.path, defaults.session.path, "TAU_SESSION_PATH");
    show_field!("session", "import_mode", config.session.import_mode, defaults.session.import_mode, "TAU_SESSION_IMPORT_MODE");

    lines.push(String::new());
    lines.push("[policy]".to_string());
    show_field!("policy", "tool_policy_preset", config.policy.tool_policy_preset, defaults.policy.tool_policy_preset, "TAU_TOOL_POLICY_PRESET");
    show_field!("policy", "bash_profile", config.policy.bash_profile, defaults.policy.bash_profile, "TAU_BASH_PROFILE");
    show_field!("policy", "os_sandbox_mode", config.policy.os_sandbox_mode, defaults.policy.os_sandbox_mode, "TAU_OS_SANDBOX_MODE");
    show_field!("policy", "bash_timeout_ms", config.policy.bash_timeout_ms, defaults.policy.bash_timeout_ms, "TAU_BASH_TIMEOUT_MS");

    lines.push(String::new());
    lines.push("[memory]".to_string());
    show_field!("memory", "action_history_enabled", config.memory.action_history_enabled, defaults.memory.action_history_enabled, "TAU_ACTION_HISTORY_ENABLED");
    show_field!("memory", "action_history_retention_days", config.memory.action_history_retention_days, defaults.memory.action_history_retention_days, "TAU_ACTION_HISTORY_RETENTION_DAYS");
    show_field!("memory", "cortex_enabled", config.memory.cortex_enabled, defaults.memory.cortex_enabled, "TAU_CORTEX_ENABLED");

    lines.push(String::new());
    lines.push("[safety]".to_string());
    show_field!("safety", "enabled", config.safety.enabled, defaults.safety.enabled, "TAU_SAFETY_ENABLED");
    show_field!("safety", "mode", config.safety.mode, defaults.safety.mode, "TAU_SAFETY_MODE");
    show_field!("safety", "secret_leak_detection", config.safety.secret_leak_detection, defaults.safety.secret_leak_detection, "TAU_SECRET_LEAK_DETECTION");

    lines.push(String::new());
    lines.push("[auth]".to_string());
    show_field!("auth", "openai_auth_mode", config.auth.openai_auth_mode, defaults.auth.openai_auth_mode, "TAU_OPENAI_AUTH_MODE");
    show_field!("auth", "anthropic_auth_mode", config.auth.anthropic_auth_mode, defaults.auth.anthropic_auth_mode, "TAU_ANTHROPIC_AUTH_MODE");

    lines.join("\n")
}

/// Generate a default `.tau.toml` and write it to disk in the current directory.
///
/// Returns a success message or an I/O error.
pub fn handle_init_auto() -> Result<String, std::io::Error> {
    let config = generate_default_config();
    std::fs::write(".tau.toml", &config)?;
    Ok("Created .tau.toml with default configuration".to_string())
}

/// Generate a default `.tau.toml` and write it to the specified path.
///
/// Returns a success message or an I/O error.
pub fn handle_init_at_path(path: &Path) -> Result<String, std::io::Error> {
    let config = generate_default_config();
    std::fs::write(path, &config)?;
    Ok(format!(
        "Created {} with default configuration",
        path.display()
    ))
}

// ---------------------------------------------------------------------------
// B3: Config self-optimization suggestions
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// A suggestion to adjust a configuration value, produced by analysing runtime
/// metrics (e.g. store fill level, retention window vs. session frequency).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOptimizationSuggestion {
    /// Dotted path to the config field (e.g. `"memory.action_history_max_records"`).
    pub field: String,
    /// The current value as a display string.
    pub current_value: String,
    /// The suggested new value.
    pub suggested_value: String,
    /// Human-readable explanation for the suggestion.
    pub rationale: String,
    /// Confidence in the suggestion, from 0.0 (speculative) to 1.0 (certain).
    pub confidence: f64,
}

/// Analyse runtime metrics and return zero or more optimisation suggestions
/// for the Tau configuration.
///
/// Current heuristics:
/// - If the action-history store is > 80% full, suggest doubling `max_records`.
/// - If the average records-per-session is high enough that the store would
///   fill within `retention_days`, suggest increasing retention or max_records.
pub fn suggest_config_optimizations(
    total_records: usize,
    max_records: usize,
    retention_days: u32,
    avg_records_per_session: f64,
) -> Vec<ConfigOptimizationSuggestion> {
    let mut suggestions = vec![];

    // Guard against division by zero.
    if max_records == 0 {
        return suggestions;
    }

    let fill_ratio = total_records as f64 / max_records as f64;

    // Heuristic 1: store is more than 80% full.
    if fill_ratio > 0.8 {
        suggestions.push(ConfigOptimizationSuggestion {
            field: "memory.action_history_max_records".to_string(),
            current_value: max_records.to_string(),
            suggested_value: (max_records * 2).to_string(),
            rationale: format!(
                "Store is {:.0}% full ({}/{})",
                fill_ratio * 100.0,
                total_records,
                max_records,
            ),
            confidence: 0.85,
        });
    }

    // Heuristic 2: at current ingestion rate the store would fill before
    // the retention window expires (assuming ~1 session/day).
    if avg_records_per_session > 0.0 && retention_days > 0 {
        let projected_records = avg_records_per_session * retention_days as f64;
        if projected_records > max_records as f64 {
            suggestions.push(ConfigOptimizationSuggestion {
                field: "memory.action_history_retention_days".to_string(),
                current_value: retention_days.to_string(),
                suggested_value: format!(
                    "{}",
                    (max_records as f64 / avg_records_per_session).floor() as u32
                ),
                rationale: format!(
                    "At {:.1} records/session, {} days of retention would produce ~{:.0} records, exceeding max_records ({})",
                    avg_records_per_session, retention_days, projected_records, max_records,
                ),
                confidence: 0.7,
            });
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_handle_config_validate_valid_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");
        std::fs::write(
            &path,
            r#"
[agent]
name = "test"
model = "test-model"
"#,
        )
        .expect("write");

        let result = handle_config_validate(&path);
        assert_eq!(result, "Configuration valid.");
    }

    #[test]
    fn test_handle_config_validate_invalid_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");
        std::fs::write(&path, "[agent\nname = broken").expect("write");

        let result = handle_config_validate(&path);
        assert!(result.starts_with("Configuration error:"));
    }

    #[test]
    fn test_handle_config_validate_missing_file() {
        let result = handle_config_validate(Path::new("/tmp/nonexistent-config-test.toml"));
        assert!(result.starts_with("Configuration error:"));
    }

    #[test]
    fn test_handle_config_show_default_values() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");
        let config = crate::config_file::generate_default_config();
        std::fs::write(&path, &config).expect("write");

        let result = handle_config_show(&path, &[]);
        assert!(result.contains("[agent]"));
        assert!(result.contains("# default"));
        assert!(result.contains("model"));
    }

    #[test]
    fn test_handle_config_show_custom_values() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");
        std::fs::write(
            &path,
            r#"
[agent]
name = "custom-agent"
model = "custom-model"
"#,
        )
        .expect("write");

        let result = handle_config_show(&path, &[]);
        assert!(result.contains("# from .tau.toml"));
        assert!(result.contains("custom-model"));
    }

    #[test]
    fn test_handle_config_show_with_env_override() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");
        let config = crate::config_file::generate_default_config();
        std::fs::write(&path, &config).expect("write");

        let overrides = vec![("TAU_MODEL".to_string(), "env-model".to_string())];
        let result = handle_config_show(&path, &overrides);
        assert!(result.contains("from env"));
        assert!(result.contains("env-model"));
    }

    #[test]
    fn test_handle_config_show_missing_file() {
        let result =
            handle_config_show(Path::new("/tmp/nonexistent-config-show.toml"), &[]);
        assert!(result.starts_with("Failed to load config:"));
    }

    #[test]
    fn test_handle_init_at_path_creates_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");

        let result = handle_init_at_path(&path);
        assert!(result.is_ok());
        assert!(path.exists());

        // Verify the generated file is valid TOML
        let contents = std::fs::read_to_string(&path).expect("read");
        let parsed = crate::config_file::parse_tau_config(&contents);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_handle_init_at_path_message() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(".tau.toml");

        let result = handle_init_at_path(&path).expect("should succeed");
        assert!(result.contains("default configuration"));
        assert!(result.contains(&path.display().to_string()));
    }

    // -------------------------------------------------------------------
    // B3: suggest_config_optimizations
    // -------------------------------------------------------------------

    #[test]
    fn suggests_increase_when_store_over_80_percent() {
        let suggestions = suggest_config_optimizations(850, 1000, 30, 10.0);
        assert!(
            suggestions
                .iter()
                .any(|s| s.field == "memory.action_history_max_records"),
            "expected max_records suggestion"
        );
        let s = suggestions
            .iter()
            .find(|s| s.field == "memory.action_history_max_records")
            .unwrap();
        assert_eq!(s.suggested_value, "2000");
        assert!(s.confidence > 0.8);
    }

    #[test]
    fn no_suggestion_when_store_under_80_percent() {
        let suggestions = suggest_config_optimizations(500, 1000, 30, 5.0);
        assert!(
            !suggestions
                .iter()
                .any(|s| s.field == "memory.action_history_max_records"),
            "should not suggest max_records increase when store is only 50% full"
        );
    }

    #[test]
    fn suggests_retention_reduction_when_projection_exceeds_max() {
        // 50 records/session * 30 days = 1500, but max is 1000
        let suggestions = suggest_config_optimizations(100, 1000, 30, 50.0);
        assert!(
            suggestions
                .iter()
                .any(|s| s.field == "memory.action_history_retention_days"),
            "expected retention suggestion"
        );
    }

    #[test]
    fn no_retention_suggestion_when_within_budget() {
        // 10 records/session * 30 days = 300, within max of 1000
        let suggestions = suggest_config_optimizations(100, 1000, 30, 10.0);
        assert!(
            !suggestions
                .iter()
                .any(|s| s.field == "memory.action_history_retention_days"),
        );
    }

    #[test]
    fn no_suggestions_when_max_records_is_zero() {
        let suggestions = suggest_config_optimizations(0, 0, 30, 10.0);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn suggestion_round_trips_through_json() {
        let suggestions = suggest_config_optimizations(900, 1000, 30, 50.0);
        assert!(!suggestions.is_empty());
        let json = serde_json::to_string(&suggestions).expect("serialize");
        let deserialized: Vec<ConfigOptimizationSuggestion> =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.len(), suggestions.len());
    }
}
