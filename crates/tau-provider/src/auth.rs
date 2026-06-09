//! Provider auth capability, mode selection, and validation helpers.
//!
//! This module resolves effective auth methods per provider and surfaces
//! capability/requirement checks used by client construction and CLI auth flows.
//! Invalid or incomplete auth configuration fails with explicit guidance.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use tau_ai::Provider;
use tau_cli::Cli;

use crate::types::{AuthCommandConfig, ProviderAuthMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public struct `ProviderAuthCapability` used across Tau components.
pub struct ProviderAuthCapability {
    pub method: ProviderAuthMethod,
    pub supported: bool,
    pub reason: &'static str,
}

const OPENAI_AUTH_CAPABILITIES: &[ProviderAuthCapability] = &[
    ProviderAuthCapability {
        method: ProviderAuthMethod::ApiKey,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::OauthToken,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::Adc,
        supported: false,
        reason: "not_implemented",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::SessionToken,
        supported: true,
        reason: "supported",
    },
];

const ANTHROPIC_AUTH_CAPABILITIES: &[ProviderAuthCapability] = &[
    ProviderAuthCapability {
        method: ProviderAuthMethod::ApiKey,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::OauthToken,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::Adc,
        supported: false,
        reason: "not_implemented",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::SessionToken,
        supported: true,
        reason: "supported",
    },
];

const GOOGLE_AUTH_CAPABILITIES: &[ProviderAuthCapability] = &[
    ProviderAuthCapability {
        method: ProviderAuthMethod::ApiKey,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::OauthToken,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::Adc,
        supported: true,
        reason: "supported",
    },
    ProviderAuthCapability {
        method: ProviderAuthMethod::SessionToken,
        supported: false,
        reason: "unsupported",
    },
];

fn provider_auth_capabilities(provider: Provider) -> &'static [ProviderAuthCapability] {
    match provider {
        Provider::OpenAi => OPENAI_AUTH_CAPABILITIES,
        Provider::OpenRouter => OPENAI_AUTH_CAPABILITIES,
        Provider::Anthropic => ANTHROPIC_AUTH_CAPABILITIES,
        Provider::Google => GOOGLE_AUTH_CAPABILITIES,
    }
}

/// Public `fn` `provider_auth_capability` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_auth_capability(
    provider: Provider,
    method: ProviderAuthMethod,
) -> ProviderAuthCapability {
    provider_auth_capabilities(provider)
        .iter()
        .find(|capability| capability.method == method)
        .copied()
        .unwrap_or(ProviderAuthCapability {
            method,
            supported: false,
            reason: "unknown",
        })
}

/// Public `fn` `provider_supported_auth_modes` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_supported_auth_modes(provider: Provider) -> Vec<ProviderAuthMethod> {
    provider_auth_capabilities(provider)
        .iter()
        .filter(|capability| capability.supported)
        .map(|capability| capability.method)
        .collect()
}

/// Public `fn` `configured_provider_auth_method` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn configured_provider_auth_method(cli: &Cli, provider: Provider) -> ProviderAuthMethod {
    match provider {
        Provider::OpenAi => cli.openai_auth_mode.into(),
        Provider::OpenRouter => cli.openai_auth_mode.into(),
        Provider::Anthropic => cli.anthropic_auth_mode.into(),
        Provider::Google => cli.google_auth_mode.into(),
    }
}

/// Public `fn` `configured_provider_auth_method_from_config` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn configured_provider_auth_method_from_config(
    config: &AuthCommandConfig,
    provider: Provider,
) -> ProviderAuthMethod {
    match provider {
        Provider::OpenAi => config.openai_auth_mode,
        Provider::OpenRouter => config.openai_auth_mode,
        Provider::Anthropic => config.anthropic_auth_mode,
        Provider::Google => config.google_auth_mode,
    }
}

/// Public `fn` `provider_auth_mode_flag` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_auth_mode_flag(provider: Provider) -> &'static str {
    match provider {
        Provider::OpenAi => "--openai-auth-mode",
        Provider::OpenRouter => "--openai-auth-mode",
        Provider::Anthropic => "--anthropic-auth-mode",
        Provider::Google => "--google-auth-mode",
    }
}

/// Public `fn` `missing_provider_api_key_message` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn missing_provider_api_key_message(provider: Provider) -> &'static str {
    match provider {
        Provider::OpenAi => {
            "missing OpenAI-compatible API key. Set OPENAI_API_KEY, OPENROUTER_API_KEY, TAU_OPENROUTER_API_KEY, DEEPSEEK_API_KEY, TAU_DEEPSEEK_API_KEY, GROQ_API_KEY, XAI_API_KEY, MISTRAL_API_KEY, AZURE_OPENAI_API_KEY, TAU_API_KEY, --openai-api-key, or --api-key"
        }
        Provider::OpenRouter => {
            "missing OpenRouter API key. Set OPENROUTER_API_KEY, TAU_OPENROUTER_API_KEY, OPENAI_API_KEY, TAU_API_KEY, --openai-api-key, or --api-key"
        }
        Provider::Anthropic => {
            "missing Anthropic API key. Set ANTHROPIC_API_KEY, TAU_API_KEY, --anthropic-api-key, or --api-key"
        }
        Provider::Google => {
            "missing Google API key. Set GEMINI_API_KEY, GOOGLE_API_KEY, TAU_API_KEY, --google-api-key, or --api-key"
        }
    }
}

/// Public `fn` `provider_api_key_candidates_with_inputs` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_api_key_candidates_with_inputs(
    provider: Provider,
    api_key: Option<String>,
    openai_api_key: Option<String>,
    anthropic_api_key: Option<String>,
    google_api_key: Option<String>,
) -> Vec<(&'static str, Option<String>)> {
    match provider {
        Provider::OpenAi => vec![
            ("--openai-api-key", openai_api_key),
            ("--api-key", api_key),
            (
                "OPENAI_API_KEY",
                provider_env_or_dotenv_var("OPENAI_API_KEY"),
            ),
            (
                "OPENROUTER_API_KEY",
                provider_env_or_dotenv_var("OPENROUTER_API_KEY"),
            ),
            (
                "TAU_OPENROUTER_API_KEY",
                provider_env_or_dotenv_var("TAU_OPENROUTER_API_KEY"),
            ),
            (
                "DEEPSEEK_API_KEY",
                provider_env_or_dotenv_var("DEEPSEEK_API_KEY"),
            ),
            (
                "TAU_DEEPSEEK_API_KEY",
                provider_env_or_dotenv_var("TAU_DEEPSEEK_API_KEY"),
            ),
            ("GROQ_API_KEY", provider_env_or_dotenv_var("GROQ_API_KEY")),
            ("XAI_API_KEY", provider_env_or_dotenv_var("XAI_API_KEY")),
            (
                "MISTRAL_API_KEY",
                provider_env_or_dotenv_var("MISTRAL_API_KEY"),
            ),
            (
                "AZURE_OPENAI_API_KEY",
                provider_env_or_dotenv_var("AZURE_OPENAI_API_KEY"),
            ),
            ("TAU_API_KEY", provider_env_or_dotenv_var("TAU_API_KEY")),
        ],
        Provider::OpenRouter => vec![
            ("--openai-api-key", openai_api_key),
            ("--api-key", api_key),
            (
                "OPENROUTER_API_KEY",
                provider_env_or_dotenv_var("OPENROUTER_API_KEY"),
            ),
            (
                "TAU_OPENROUTER_API_KEY",
                provider_env_or_dotenv_var("TAU_OPENROUTER_API_KEY"),
            ),
            (
                "OPENAI_API_KEY",
                provider_env_or_dotenv_var("OPENAI_API_KEY"),
            ),
            ("TAU_API_KEY", provider_env_or_dotenv_var("TAU_API_KEY")),
        ],
        Provider::Anthropic => vec![
            ("--anthropic-api-key", anthropic_api_key),
            ("--api-key", api_key),
            (
                "ANTHROPIC_API_KEY",
                provider_env_or_dotenv_var("ANTHROPIC_API_KEY"),
            ),
            ("TAU_API_KEY", provider_env_or_dotenv_var("TAU_API_KEY")),
        ],
        Provider::Google => vec![
            ("--google-api-key", google_api_key),
            ("--api-key", api_key),
            (
                "GEMINI_API_KEY",
                provider_env_or_dotenv_var("GEMINI_API_KEY"),
            ),
            (
                "GOOGLE_API_KEY",
                provider_env_or_dotenv_var("GOOGLE_API_KEY"),
            ),
            ("TAU_API_KEY", provider_env_or_dotenv_var("TAU_API_KEY")),
        ],
    }
}

fn provider_env_or_dotenv_var(name: &str) -> Option<String> {
    if let Ok(value) = std::env::var(name) {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    if provider_dotenv_keys_disabled() {
        return None;
    }
    nearest_dotenv_var(name)
}

fn provider_dotenv_keys_disabled() -> bool {
    std::env::var("TAU_PROVIDER_DISABLE_DOTENV_KEYS")
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn nearest_dotenv_var(name: &str) -> Option<String> {
    let current_dir = std::env::current_dir().ok()?;
    for dir in current_dir.ancestors() {
        let dotenv_path = dir.join(".env");
        if let Some(value) = dotenv_var_from_path(&dotenv_path, name) {
            return Some(value);
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    None
}

fn dotenv_var_from_path(path: &Path, name: &str) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    contents
        .lines()
        .filter_map(|line| parse_dotenv_line(line, name))
        .find(|value| !value.trim().is_empty())
}

fn parse_dotenv_line(line: &str, expected_name: &str) -> Option<String> {
    let mut trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("export ") {
        trimmed = rest.trim_start();
    }
    let (name, value) = trimmed.split_once('=')?;
    let name = name.trim();
    if name != expected_name || !is_dotenv_name(name) {
        return None;
    }
    Some(strip_dotenv_quotes(value.trim()))
}

fn is_dotenv_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn strip_dotenv_quotes(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

/// Public `fn` `provider_api_key_candidates` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_api_key_candidates(
    cli: &Cli,
    provider: Provider,
) -> Vec<(&'static str, Option<String>)> {
    provider_api_key_candidates_with_inputs(
        provider,
        cli.api_key.clone(),
        cli.openai_api_key.clone(),
        cli.anthropic_api_key.clone(),
        cli.google_api_key.clone(),
    )
}

/// Public `fn` `resolve_api_key` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn resolve_api_key(candidates: Vec<Option<String>>) -> Option<String> {
    candidates
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

/// Public `fn` `provider_api_key_candidates_from_auth_config` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_api_key_candidates_from_auth_config(
    config: &AuthCommandConfig,
    provider: Provider,
) -> Vec<(&'static str, Option<String>)> {
    provider_api_key_candidates_with_inputs(
        provider,
        config.api_key.clone(),
        config.openai_api_key.clone(),
        config.anthropic_api_key.clone(),
        config.google_api_key.clone(),
    )
}

/// Public `fn` `provider_login_access_token_candidates` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_login_access_token_candidates(
    provider: Provider,
) -> Vec<(&'static str, Option<String>)> {
    match provider {
        Provider::OpenAi => vec![
            (
                "TAU_AUTH_ACCESS_TOKEN",
                std::env::var("TAU_AUTH_ACCESS_TOKEN").ok(),
            ),
            (
                "OPENAI_ACCESS_TOKEN",
                std::env::var("OPENAI_ACCESS_TOKEN").ok(),
            ),
        ],
        Provider::OpenRouter => vec![
            (
                "TAU_AUTH_ACCESS_TOKEN",
                std::env::var("TAU_AUTH_ACCESS_TOKEN").ok(),
            ),
            (
                "OPENAI_ACCESS_TOKEN",
                std::env::var("OPENAI_ACCESS_TOKEN").ok(),
            ),
        ],
        Provider::Anthropic => vec![
            (
                "TAU_AUTH_ACCESS_TOKEN",
                std::env::var("TAU_AUTH_ACCESS_TOKEN").ok(),
            ),
            (
                "ANTHROPIC_ACCESS_TOKEN",
                std::env::var("ANTHROPIC_ACCESS_TOKEN").ok(),
            ),
        ],
        Provider::Google => vec![
            (
                "TAU_AUTH_ACCESS_TOKEN",
                std::env::var("TAU_AUTH_ACCESS_TOKEN").ok(),
            ),
            (
                "GOOGLE_ACCESS_TOKEN",
                std::env::var("GOOGLE_ACCESS_TOKEN").ok(),
            ),
        ],
    }
}

/// Public `fn` `provider_login_refresh_token_candidates` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_login_refresh_token_candidates(
    provider: Provider,
) -> Vec<(&'static str, Option<String>)> {
    match provider {
        Provider::OpenAi => vec![
            (
                "TAU_AUTH_REFRESH_TOKEN",
                std::env::var("TAU_AUTH_REFRESH_TOKEN").ok(),
            ),
            (
                "OPENAI_REFRESH_TOKEN",
                std::env::var("OPENAI_REFRESH_TOKEN").ok(),
            ),
        ],
        Provider::OpenRouter => vec![
            (
                "TAU_AUTH_REFRESH_TOKEN",
                std::env::var("TAU_AUTH_REFRESH_TOKEN").ok(),
            ),
            (
                "OPENAI_REFRESH_TOKEN",
                std::env::var("OPENAI_REFRESH_TOKEN").ok(),
            ),
        ],
        Provider::Anthropic => vec![
            (
                "TAU_AUTH_REFRESH_TOKEN",
                std::env::var("TAU_AUTH_REFRESH_TOKEN").ok(),
            ),
            (
                "ANTHROPIC_REFRESH_TOKEN",
                std::env::var("ANTHROPIC_REFRESH_TOKEN").ok(),
            ),
        ],
        Provider::Google => vec![
            (
                "TAU_AUTH_REFRESH_TOKEN",
                std::env::var("TAU_AUTH_REFRESH_TOKEN").ok(),
            ),
            (
                "GOOGLE_REFRESH_TOKEN",
                std::env::var("GOOGLE_REFRESH_TOKEN").ok(),
            ),
        ],
    }
}

/// Public `fn` `provider_login_expires_candidates` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn provider_login_expires_candidates(
    provider: Provider,
) -> Vec<(&'static str, Option<String>)> {
    match provider {
        Provider::OpenAi => vec![
            (
                "TAU_AUTH_EXPIRES_UNIX",
                std::env::var("TAU_AUTH_EXPIRES_UNIX").ok(),
            ),
            (
                "OPENAI_AUTH_EXPIRES_UNIX",
                std::env::var("OPENAI_AUTH_EXPIRES_UNIX").ok(),
            ),
        ],
        Provider::OpenRouter => vec![
            (
                "TAU_AUTH_EXPIRES_UNIX",
                std::env::var("TAU_AUTH_EXPIRES_UNIX").ok(),
            ),
            (
                "OPENAI_AUTH_EXPIRES_UNIX",
                std::env::var("OPENAI_AUTH_EXPIRES_UNIX").ok(),
            ),
        ],
        Provider::Anthropic => vec![
            (
                "TAU_AUTH_EXPIRES_UNIX",
                std::env::var("TAU_AUTH_EXPIRES_UNIX").ok(),
            ),
            (
                "ANTHROPIC_AUTH_EXPIRES_UNIX",
                std::env::var("ANTHROPIC_AUTH_EXPIRES_UNIX").ok(),
            ),
        ],
        Provider::Google => vec![
            (
                "TAU_AUTH_EXPIRES_UNIX",
                std::env::var("TAU_AUTH_EXPIRES_UNIX").ok(),
            ),
            (
                "GOOGLE_AUTH_EXPIRES_UNIX",
                std::env::var("GOOGLE_AUTH_EXPIRES_UNIX").ok(),
            ),
        ],
    }
}

/// Public `fn` `resolve_auth_login_expires_unix` in `tau-provider`.
///
/// This item is part of the Wave 2 API surface for M23 documentation uplift.
/// Callers rely on its contract and failure semantics remaining stable.
/// Update this comment if behavior or integration expectations change.
pub fn resolve_auth_login_expires_unix(provider: Provider) -> Result<Option<u64>> {
    for (source, value) in provider_login_expires_candidates(provider) {
        let Some(value) = value else {
            continue;
        };
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed = trimmed
            .parse::<u64>()
            .with_context(|| format!("invalid unix timestamp in {}", source))?;
        return Ok(Some(parsed));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Mutex};

    use super::provider_api_key_candidates_with_inputs;
    use tau_ai::Provider;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn unit_provider_api_key_candidates_include_deepseek_env_vars_for_openai() {
        let candidates =
            provider_api_key_candidates_with_inputs(Provider::OpenAi, None, None, None, None);
        let names = candidates
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<&str>>();
        assert!(
            names.contains(&"DEEPSEEK_API_KEY"),
            "DEEPSEEK_API_KEY should be a valid OpenAI-compatible key source"
        );
        assert!(
            names.contains(&"TAU_DEEPSEEK_API_KEY"),
            "TAU_DEEPSEEK_API_KEY should be a valid OpenAI-compatible key source"
        );
    }

    #[test]
    fn unit_provider_api_key_candidates_read_openrouter_key_from_repo_dotenv() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        let nested = repo.join("crates/tau-provider");
        fs::create_dir_all(repo.join(".git")).expect("create fake git dir");
        fs::create_dir_all(&nested).expect("create nested dir");
        fs::write(
            repo.join(".env"),
            "OPENROUTER_API_KEY=file-openrouter-key\n",
        )
        .expect("write dotenv");
        let prior_dir = std::env::current_dir().expect("current dir");
        let prior = clear_env_vars(&[
            "OPENROUTER_API_KEY",
            "TAU_OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "TAU_API_KEY",
        ]);

        std::env::set_current_dir(&nested).expect("set current dir");
        let candidates =
            provider_api_key_candidates_with_inputs(Provider::OpenRouter, None, None, None, None);

        std::env::set_current_dir(prior_dir).expect("restore current dir");
        restore_env_vars(prior);

        assert_eq!(
            candidate_value(&candidates, "OPENROUTER_API_KEY").as_deref(),
            Some("file-openrouter-key")
        );
    }

    #[test]
    fn unit_provider_api_key_candidates_keep_process_env_precedence_over_dotenv() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join(".git")).expect("create fake git dir");
        fs::write(
            temp.path().join(".env"),
            "export OPENROUTER_API_KEY='file-openrouter-key'\n",
        )
        .expect("write dotenv");
        let prior_dir = std::env::current_dir().expect("current dir");
        let prior = clear_env_vars(&[
            "OPENROUTER_API_KEY",
            "TAU_OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "TAU_API_KEY",
        ]);

        std::env::set_var("OPENROUTER_API_KEY", "env-openrouter-key");
        std::env::set_current_dir(temp.path()).expect("set current dir");
        let candidates =
            provider_api_key_candidates_with_inputs(Provider::OpenRouter, None, None, None, None);

        std::env::set_current_dir(prior_dir).expect("restore current dir");
        restore_env_vars(prior);

        assert_eq!(
            candidate_value(&candidates, "OPENROUTER_API_KEY").as_deref(),
            Some("env-openrouter-key")
        );
    }

    #[test]
    fn unit_provider_api_key_candidates_can_disable_dotenv_fallback() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join(".git")).expect("create fake git dir");
        fs::write(
            temp.path().join(".env"),
            "OPENROUTER_API_KEY=file-openrouter-key\n",
        )
        .expect("write dotenv");
        let prior_dir = std::env::current_dir().expect("current dir");
        let prior = clear_env_vars(&[
            "OPENROUTER_API_KEY",
            "TAU_OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "TAU_API_KEY",
            "TAU_PROVIDER_DISABLE_DOTENV_KEYS",
        ]);

        std::env::set_var("TAU_PROVIDER_DISABLE_DOTENV_KEYS", "1");
        std::env::set_current_dir(temp.path()).expect("set current dir");
        let candidates =
            provider_api_key_candidates_with_inputs(Provider::OpenRouter, None, None, None, None);

        std::env::set_current_dir(prior_dir).expect("restore current dir");
        restore_env_vars(prior);

        assert!(candidate_value(&candidates, "OPENROUTER_API_KEY").is_none());
    }

    #[test]
    fn unit_provider_api_key_candidates_absent_dotenv_preserves_missing_values() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join(".git")).expect("create fake git dir");
        let prior_dir = std::env::current_dir().expect("current dir");
        let prior = clear_env_vars(&[
            "OPENROUTER_API_KEY",
            "TAU_OPENROUTER_API_KEY",
            "OPENAI_API_KEY",
            "TAU_API_KEY",
        ]);

        std::env::set_current_dir(temp.path()).expect("set current dir");
        let candidates =
            provider_api_key_candidates_with_inputs(Provider::OpenRouter, None, None, None, None);

        std::env::set_current_dir(prior_dir).expect("restore current dir");
        restore_env_vars(prior);

        assert!(candidate_value(&candidates, "OPENROUTER_API_KEY").is_none());
        assert!(candidate_value(&candidates, "TAU_OPENROUTER_API_KEY").is_none());
        assert!(candidate_value(&candidates, "OPENAI_API_KEY").is_none());
    }

    fn candidate_value(
        candidates: &[(&'static str, Option<String>)],
        name: &'static str,
    ) -> Option<String> {
        candidates
            .iter()
            .find(|(candidate_name, _)| *candidate_name == name)
            .and_then(|(_, value)| value.clone())
    }

    fn clear_env_vars(names: &[&'static str]) -> Vec<(&'static str, Option<String>)> {
        names
            .iter()
            .map(|name| {
                let prior = std::env::var(name).ok();
                std::env::remove_var(name);
                (*name, prior)
            })
            .collect()
    }

    fn restore_env_vars(prior: Vec<(&'static str, Option<String>)>) {
        for (name, value) in prior {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}
