use std::path::PathBuf;

use crate::extension_manifest::ExtensionRegisteredCommand;
use crate::{default_provider_auth_method, Cli, ModelCatalog, ProviderAuthMethod};
use serde::{Deserialize, Serialize};
pub(crate) use tau_diagnostics::DoctorCommandConfig;
#[cfg(test)]
pub(crate) use tau_diagnostics::{DoctorMultiChannelReadinessConfig, DoctorProviderKeyStatus};
pub(crate) use tau_provider::AuthCommandConfig;
use tau_session::SessionImportMode;

#[derive(Debug, Clone)]
pub(crate) struct SkillsSyncCommandConfig {
    pub(crate) skills_dir: PathBuf,
    pub(crate) default_lock_path: PathBuf,
    pub(crate) default_trust_root_path: Option<PathBuf>,
    pub(crate) doctor_config: DoctorCommandConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProfileSessionDefaults {
    pub(crate) enabled: bool,
    pub(crate) path: Option<String>,
    pub(crate) import_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProfilePolicyDefaults {
    pub(crate) tool_policy_preset: String,
    pub(crate) bash_profile: String,
    pub(crate) bash_dry_run: bool,
    pub(crate) os_sandbox_mode: String,
    pub(crate) enforce_regular_files: bool,
    pub(crate) bash_timeout_ms: u64,
    pub(crate) max_command_length: usize,
    pub(crate) max_tool_output_bytes: usize,
    pub(crate) max_file_read_bytes: usize,
    pub(crate) max_file_write_bytes: usize,
    pub(crate) allow_command_newlines: bool,
}

fn default_profile_mcp_context_providers() -> Vec<String> {
    vec![
        "session".to_string(),
        "skills".to_string(),
        "channel-store".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProfileMcpDefaults {
    #[serde(default = "default_profile_mcp_context_providers")]
    pub(crate) context_providers: Vec<String>,
}

impl Default for ProfileMcpDefaults {
    fn default() -> Self {
        Self {
            context_providers: default_profile_mcp_context_providers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProfileAuthDefaults {
    #[serde(default = "default_provider_auth_method")]
    pub(crate) openai: ProviderAuthMethod,
    #[serde(default = "default_provider_auth_method")]
    pub(crate) anthropic: ProviderAuthMethod,
    #[serde(default = "default_provider_auth_method")]
    pub(crate) google: ProviderAuthMethod,
}

impl Default for ProfileAuthDefaults {
    fn default() -> Self {
        Self {
            openai: default_provider_auth_method(),
            anthropic: default_provider_auth_method(),
            google: default_provider_auth_method(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProfileDefaults {
    pub(crate) model: String,
    pub(crate) fallback_models: Vec<String>,
    pub(crate) session: ProfileSessionDefaults,
    pub(crate) policy: ProfilePolicyDefaults,
    #[serde(default)]
    pub(crate) mcp: ProfileMcpDefaults,
    #[serde(default)]
    pub(crate) auth: ProfileAuthDefaults,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderOptions {
    pub(crate) stream_output: bool,
    pub(crate) stream_delay_ms: u64,
}

impl RenderOptions {
    pub(crate) fn from_cli(cli: &Cli) -> Self {
        Self {
            stream_output: cli.stream_output,
            stream_delay_ms: cli.stream_delay_ms,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CommandExecutionContext<'a> {
    pub(crate) tool_policy_json: &'a serde_json::Value,
    pub(crate) session_import_mode: SessionImportMode,
    pub(crate) profile_defaults: &'a ProfileDefaults,
    pub(crate) skills_command_config: &'a SkillsSyncCommandConfig,
    pub(crate) auth_command_config: &'a AuthCommandConfig,
    pub(crate) model_catalog: &'a ModelCatalog,
    pub(crate) extension_commands: &'a [ExtensionRegisteredCommand],
}
