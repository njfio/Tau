//! Discord runtime foundation contracts for Tau.
//!
//! This crate defines a minimal bootstrap boundary for Discord-specific runtime
//! wiring while preserving the existing multi-channel behavior surface.

use serde::{Deserialize, Serialize};
use serenity::all::GatewayIntents;
use thiserror::Error;
use tracing::{info, instrument};

/// Public `const` `TAU_DISCORD_RUNTIME_SCHEMA_VERSION` in `tau-discord-runtime`.
pub const TAU_DISCORD_RUNTIME_SCHEMA_VERSION: u32 = 1;

/// Public struct `DiscordRuntimeBootstrapConfig` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscordRuntimeBootstrapConfig {
    #[serde(default = "discord_runtime_schema_version")]
    pub schema_version: u32,
    pub application_id: u64,
    #[serde(default = "default_command_prefix")]
    pub command_prefix: String,
    #[serde(default = "default_gateway_intents_bits")]
    pub intents_bits: u64,
}

impl Default for DiscordRuntimeBootstrapConfig {
    fn default() -> Self {
        Self {
            schema_version: discord_runtime_schema_version(),
            application_id: 1,
            command_prefix: default_command_prefix(),
            intents_bits: default_gateway_intents_bits(),
        }
    }
}

/// Public enum `DiscordRuntimeConfigError` used across Tau components.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiscordRuntimeConfigError {
    #[error("unsupported schema_version {found} (expected {expected})")]
    UnsupportedSchema { found: u32, expected: u32 },
    #[error("application_id must be greater than 0")]
    MissingApplicationId,
    #[error("command_prefix must not be empty")]
    EmptyCommandPrefix,
    #[error("gateway intents must not be empty")]
    EmptyGatewayIntents,
}

fn discord_runtime_schema_version() -> u32 {
    TAU_DISCORD_RUNTIME_SCHEMA_VERSION
}

fn default_command_prefix() -> String {
    "/tau".to_string()
}

fn default_gateway_intents_bits() -> u64 {
    default_gateway_intents().bits()
}

/// Public `fn` `default_gateway_intents` in `tau-discord-runtime`.
#[instrument(level = "info")]
pub fn default_gateway_intents() -> GatewayIntents {
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    info!(
        intents_bits = intents.bits(),
        "resolved default Discord gateway intents"
    );
    intents
}

impl DiscordRuntimeBootstrapConfig {
    /// Public `fn` `gateway_intents` in `tau-discord-runtime`.
    pub fn gateway_intents(&self) -> GatewayIntents {
        GatewayIntents::from_bits_retain(self.intents_bits)
    }

    /// Public `fn` `validate` in `tau-discord-runtime`.
    #[instrument(level = "info", skip(self))]
    pub fn validate(&self) -> Result<(), DiscordRuntimeConfigError> {
        if self.schema_version != TAU_DISCORD_RUNTIME_SCHEMA_VERSION {
            return Err(DiscordRuntimeConfigError::UnsupportedSchema {
                found: self.schema_version,
                expected: TAU_DISCORD_RUNTIME_SCHEMA_VERSION,
            });
        }
        if self.application_id == 0 {
            return Err(DiscordRuntimeConfigError::MissingApplicationId);
        }
        if self.command_prefix.trim().is_empty() {
            return Err(DiscordRuntimeConfigError::EmptyCommandPrefix);
        }
        if self.intents_bits == 0 {
            return Err(DiscordRuntimeConfigError::EmptyGatewayIntents);
        }
        Ok(())
    }
}

/// Public `fn` `render_bootstrap_summary` in `tau-discord-runtime`.
#[instrument(level = "info", skip(config))]
pub fn render_bootstrap_summary(
    config: &DiscordRuntimeBootstrapConfig,
) -> Result<String, DiscordRuntimeConfigError> {
    config.validate()?;
    let summary = format!(
        "discord runtime bootstrap: schema_version={} application_id={} command_prefix={} intents_bits={}",
        config.schema_version,
        config.application_id,
        config.command_prefix.trim(),
        config.intents_bits
    );
    info!("rendered discord runtime bootstrap summary");
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::{
        default_gateway_intents, render_bootstrap_summary, DiscordRuntimeBootstrapConfig,
        DiscordRuntimeConfigError, TAU_DISCORD_RUNTIME_SCHEMA_VERSION,
    };

    #[test]
    fn unit_validate_rejects_zero_application_id() {
        let config = DiscordRuntimeBootstrapConfig {
            application_id: 0,
            ..DiscordRuntimeBootstrapConfig::default()
        };
        let error = config
            .validate()
            .expect_err("zero application id must fail");
        assert_eq!(error, DiscordRuntimeConfigError::MissingApplicationId);
    }

    #[test]
    fn unit_validate_rejects_blank_command_prefix() {
        let config = DiscordRuntimeBootstrapConfig {
            command_prefix: "   ".to_string(),
            ..DiscordRuntimeBootstrapConfig::default()
        };
        let error = config.validate().expect_err("blank prefix must fail");
        assert_eq!(error, DiscordRuntimeConfigError::EmptyCommandPrefix);
    }

    #[test]
    fn unit_validate_rejects_empty_gateway_intents() {
        let config = DiscordRuntimeBootstrapConfig {
            intents_bits: 0,
            ..DiscordRuntimeBootstrapConfig::default()
        };
        let error = config.validate().expect_err("empty intents must fail");
        assert_eq!(error, DiscordRuntimeConfigError::EmptyGatewayIntents);
    }

    #[test]
    fn unit_validate_rejects_unsupported_schema_version() {
        let config = DiscordRuntimeBootstrapConfig {
            schema_version: TAU_DISCORD_RUNTIME_SCHEMA_VERSION.saturating_add(1),
            ..DiscordRuntimeBootstrapConfig::default()
        };
        let error = config
            .validate()
            .expect_err("unsupported schema version must fail");
        assert_eq!(
            error,
            DiscordRuntimeConfigError::UnsupportedSchema {
                found: TAU_DISCORD_RUNTIME_SCHEMA_VERSION.saturating_add(1),
                expected: TAU_DISCORD_RUNTIME_SCHEMA_VERSION,
            }
        );
    }

    #[test]
    fn functional_render_bootstrap_summary_includes_stable_fields() {
        let config = DiscordRuntimeBootstrapConfig::default();
        let summary = render_bootstrap_summary(&config).expect("render summary");
        assert!(summary.contains("discord runtime bootstrap:"));
        assert!(summary.contains("command_prefix=/tau"));
        assert!(summary.contains("intents_bits="));
    }

    #[test]
    fn regression_default_gateway_intents_remain_non_empty() {
        let intents = default_gateway_intents();
        assert_ne!(intents.bits(), 0);
        assert!(intents.contains(serenity::all::GatewayIntents::MESSAGE_CONTENT));
    }
}
