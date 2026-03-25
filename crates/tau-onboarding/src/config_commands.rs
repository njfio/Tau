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
}
