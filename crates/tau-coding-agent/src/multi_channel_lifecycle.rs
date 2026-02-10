use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::credentials::{load_credential_store, resolve_non_empty_cli_value};
use crate::multi_channel_contract::MultiChannelTransport;
use crate::{
    current_unix_timestamp_ms, resolve_credential_store_encryption_mode, write_text_atomic, Cli,
};

pub(crate) const MULTI_CHANNEL_LIFECYCLE_STATE_FILE_NAME: &str = "channel-lifecycle.json";
const MULTI_CHANNEL_LIFECYCLE_STATE_SCHEMA_VERSION: u32 = 1;

const TELEGRAM_TOKEN_INTEGRATION_ID: &str = "telegram-bot-token";
const DISCORD_TOKEN_INTEGRATION_ID: &str = "discord-bot-token";
const WHATSAPP_TOKEN_INTEGRATION_ID: &str = "whatsapp-access-token";
const WHATSAPP_PHONE_NUMBER_ID_INTEGRATION_ID: &str = "whatsapp-phone-number-id";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecycleActionKind {
    Status,
    Login,
    Logout,
    Probe,
}

impl LifecycleActionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::Probe => "probe",
        }
    }
}

#[derive(Debug, Clone)]
struct MultiChannelLifecycleCommandConfig {
    state_dir: PathBuf,
    ingress_dir: PathBuf,
    credential_store_path: PathBuf,
    credential_store_encryption: crate::CredentialStoreEncryptionMode,
    credential_store_key: Option<String>,
    telegram_bot_token: Option<String>,
    discord_bot_token: Option<String>,
    whatsapp_access_token: Option<String>,
    whatsapp_phone_number_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct MultiChannelLifecycleReport {
    pub(crate) action: String,
    pub(crate) channel: String,
    pub(crate) lifecycle_status: String,
    pub(crate) readiness_status: String,
    pub(crate) reason_codes: Vec<String>,
    pub(crate) ingress_file: String,
    pub(crate) ingress_exists: bool,
    pub(crate) ingress_is_file: bool,
    pub(crate) token_source: String,
    pub(crate) phone_number_source: String,
    pub(crate) state_path: String,
    pub(crate) state_persisted: bool,
    pub(crate) updated_unix_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MultiChannelLifecycleStateFile {
    #[serde(default = "multi_channel_lifecycle_state_schema_version")]
    schema_version: u32,
    #[serde(default)]
    channels: BTreeMap<String, MultiChannelLifecycleChannelState>,
}

impl Default for MultiChannelLifecycleStateFile {
    fn default() -> Self {
        Self {
            schema_version: MULTI_CHANNEL_LIFECYCLE_STATE_SCHEMA_VERSION,
            channels: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct MultiChannelLifecycleChannelState {
    #[serde(default)]
    lifecycle_status: String,
    #[serde(default)]
    reason_codes: Vec<String>,
    #[serde(default)]
    last_action: String,
    #[serde(default)]
    last_updated_unix_ms: u64,
    #[serde(default)]
    last_login_unix_ms: u64,
    #[serde(default)]
    last_logout_unix_ms: u64,
    #[serde(default)]
    last_probe_unix_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct ResolvedSecret {
    value: Option<String>,
    source: String,
    credential_store_unreadable: bool,
}

#[derive(Debug, Clone)]
struct ChannelReadiness {
    readiness_status: String,
    reason_codes: Vec<String>,
    ingress_file: PathBuf,
    ingress_exists: bool,
    ingress_is_file: bool,
    token_source: String,
    phone_number_source: String,
}

fn multi_channel_lifecycle_state_schema_version() -> u32 {
    MULTI_CHANNEL_LIFECYCLE_STATE_SCHEMA_VERSION
}

pub(crate) fn execute_multi_channel_channel_lifecycle_command(cli: &Cli) -> Result<()> {
    let Some((action, channel, json_output)) = lifecycle_action_from_cli(cli) else {
        return Ok(());
    };
    let config = build_multi_channel_lifecycle_command_config(cli);
    let report = execute_multi_channel_lifecycle_action(&config, action, channel)?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .context("failed to render multi-channel lifecycle json")?
        );
    } else {
        println!("{}", render_multi_channel_lifecycle_report(&report));
    }
    Ok(())
}

fn build_multi_channel_lifecycle_command_config(cli: &Cli) -> MultiChannelLifecycleCommandConfig {
    MultiChannelLifecycleCommandConfig {
        state_dir: cli.multi_channel_state_dir.clone(),
        ingress_dir: cli.multi_channel_live_ingress_dir.clone(),
        credential_store_path: cli.credential_store.clone(),
        credential_store_encryption: resolve_credential_store_encryption_mode(cli),
        credential_store_key: cli.credential_store_key.clone(),
        telegram_bot_token: resolve_non_empty_cli_value(
            cli.multi_channel_telegram_bot_token.as_deref(),
        ),
        discord_bot_token: resolve_non_empty_cli_value(
            cli.multi_channel_discord_bot_token.as_deref(),
        ),
        whatsapp_access_token: resolve_non_empty_cli_value(
            cli.multi_channel_whatsapp_access_token.as_deref(),
        ),
        whatsapp_phone_number_id: resolve_non_empty_cli_value(
            cli.multi_channel_whatsapp_phone_number_id.as_deref(),
        ),
    }
}

fn lifecycle_action_from_cli(
    cli: &Cli,
) -> Option<(LifecycleActionKind, MultiChannelTransport, bool)> {
    if let Some(channel) = cli.multi_channel_channel_status {
        return Some((
            LifecycleActionKind::Status,
            channel.into(),
            cli.multi_channel_channel_status_json,
        ));
    }
    if let Some(channel) = cli.multi_channel_channel_login {
        return Some((
            LifecycleActionKind::Login,
            channel.into(),
            cli.multi_channel_channel_login_json,
        ));
    }
    if let Some(channel) = cli.multi_channel_channel_logout {
        return Some((
            LifecycleActionKind::Logout,
            channel.into(),
            cli.multi_channel_channel_logout_json,
        ));
    }
    if let Some(channel) = cli.multi_channel_channel_probe {
        return Some((
            LifecycleActionKind::Probe,
            channel.into(),
            cli.multi_channel_channel_probe_json,
        ));
    }
    None
}

fn execute_multi_channel_lifecycle_action(
    config: &MultiChannelLifecycleCommandConfig,
    action: LifecycleActionKind,
    channel: MultiChannelTransport,
) -> Result<MultiChannelLifecycleReport> {
    let state_path = lifecycle_state_path_for_dir(&config.state_dir);
    let mut state = load_multi_channel_lifecycle_state(&state_path)?;
    let channel_key = channel.as_str().to_string();
    let existing_entry = state
        .channels
        .get(&channel_key)
        .cloned()
        .unwrap_or_default();
    let mut readiness = probe_channel_readiness(
        config,
        channel,
        !matches!(action, LifecycleActionKind::Login),
    );

    let now_unix_ms = current_unix_timestamp_ms();
    let mut lifecycle_status = if existing_entry.lifecycle_status.trim().is_empty() {
        "unknown".to_string()
    } else {
        existing_entry.lifecycle_status.clone()
    };
    let mut reason_codes = readiness.reason_codes.clone();
    let mut state_persisted = false;

    match action {
        LifecycleActionKind::Status => {}
        LifecycleActionKind::Login => {
            if readiness.readiness_status == "pass" {
                ensure_ingress_file_exists(&readiness.ingress_file)?;
                readiness.ingress_exists = true;
                readiness.ingress_is_file = true;
                readiness.readiness_status = "pass".to_string();
                readiness.reason_codes = vec!["ready".to_string()];
                reason_codes = readiness.reason_codes.clone();
                lifecycle_status = "initialized".to_string();
            } else {
                lifecycle_status = "login_failed".to_string();
            }
            let entry = state.channels.entry(channel_key).or_default();
            entry.lifecycle_status = lifecycle_status.clone();
            entry.reason_codes = reason_codes.clone();
            entry.last_action = action.as_str().to_string();
            entry.last_updated_unix_ms = now_unix_ms;
            entry.last_login_unix_ms = now_unix_ms;
            save_multi_channel_lifecycle_state(&state_path, &state)?;
            state_persisted = true;
        }
        LifecycleActionKind::Logout => {
            lifecycle_status = "logged_out".to_string();
            reason_codes = vec!["logout_requested".to_string()];
            let entry = state.channels.entry(channel_key).or_default();
            entry.lifecycle_status = lifecycle_status.clone();
            entry.reason_codes = reason_codes.clone();
            entry.last_action = action.as_str().to_string();
            entry.last_updated_unix_ms = now_unix_ms;
            entry.last_logout_unix_ms = now_unix_ms;
            save_multi_channel_lifecycle_state(&state_path, &state)?;
            state_persisted = true;
        }
        LifecycleActionKind::Probe => {
            lifecycle_status = if readiness.readiness_status == "pass" {
                "ready".to_string()
            } else {
                "probe_failed".to_string()
            };
            reason_codes = readiness.reason_codes.clone();
            let entry = state.channels.entry(channel_key).or_default();
            entry.lifecycle_status = lifecycle_status.clone();
            entry.reason_codes = reason_codes.clone();
            entry.last_action = action.as_str().to_string();
            entry.last_updated_unix_ms = now_unix_ms;
            entry.last_probe_unix_ms = now_unix_ms;
            save_multi_channel_lifecycle_state(&state_path, &state)?;
            state_persisted = true;
        }
    }

    Ok(MultiChannelLifecycleReport {
        action: action.as_str().to_string(),
        channel: channel.as_str().to_string(),
        lifecycle_status,
        readiness_status: readiness.readiness_status,
        reason_codes,
        ingress_file: readiness.ingress_file.display().to_string(),
        ingress_exists: readiness.ingress_exists,
        ingress_is_file: readiness.ingress_is_file,
        token_source: readiness.token_source,
        phone_number_source: readiness.phone_number_source,
        state_path: state_path.display().to_string(),
        state_persisted,
        updated_unix_ms: now_unix_ms,
    })
}

fn lifecycle_state_path_for_dir(state_dir: &Path) -> PathBuf {
    state_dir
        .join("security")
        .join(MULTI_CHANNEL_LIFECYCLE_STATE_FILE_NAME)
}

fn load_multi_channel_lifecycle_state(path: &Path) -> Result<MultiChannelLifecycleStateFile> {
    if !path.exists() {
        return Ok(MultiChannelLifecycleStateFile::default());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: MultiChannelLifecycleStateFile = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse multi-channel lifecycle state {}",
            path.display()
        )
    })?;
    if parsed.schema_version != MULTI_CHANNEL_LIFECYCLE_STATE_SCHEMA_VERSION {
        bail!(
            "unsupported multi-channel lifecycle schema {} in {}",
            parsed.schema_version,
            path.display()
        );
    }
    Ok(parsed)
}

fn save_multi_channel_lifecycle_state(
    path: &Path,
    state: &MultiChannelLifecycleStateFile,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    let payload =
        serde_json::to_string_pretty(state).context("failed to serialize lifecycle state")?;
    write_text_atomic(path, &payload).with_context(|| format!("failed to write {}", path.display()))
}

fn ensure_ingress_file_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    if path.exists() && !path.is_file() {
        bail!("ingress path '{}' exists but is not a file", path.display());
    }
    if !path.exists() {
        std::fs::write(path, "").with_context(|| format!("failed to create {}", path.display()))?;
    }
    Ok(())
}

fn ingress_file_for_transport(ingress_dir: &Path, channel: MultiChannelTransport) -> PathBuf {
    ingress_dir.join(format!("{}.ndjson", channel.as_str()))
}

fn probe_channel_readiness(
    config: &MultiChannelLifecycleCommandConfig,
    channel: MultiChannelTransport,
    require_ingress_file: bool,
) -> ChannelReadiness {
    let ingress_file = ingress_file_for_transport(&config.ingress_dir, channel);
    let ingress_exists = ingress_file.exists();
    let ingress_is_file = ingress_file.is_file();
    let mut reason_codes = Vec::new();
    let (token_source, phone_number_source) = match channel {
        MultiChannelTransport::Telegram => {
            let token = resolve_lifecycle_secret(
                config,
                config.telegram_bot_token.as_deref(),
                TELEGRAM_TOKEN_INTEGRATION_ID,
            );
            if token.credential_store_unreadable {
                reason_codes.push("credential_store_unreadable".to_string());
            }
            if token.value.is_none() {
                reason_codes.push("missing_telegram_bot_token".to_string());
            }
            (token.source, "not_required".to_string())
        }
        MultiChannelTransport::Discord => {
            let token = resolve_lifecycle_secret(
                config,
                config.discord_bot_token.as_deref(),
                DISCORD_TOKEN_INTEGRATION_ID,
            );
            if token.credential_store_unreadable {
                reason_codes.push("credential_store_unreadable".to_string());
            }
            if token.value.is_none() {
                reason_codes.push("missing_discord_bot_token".to_string());
            }
            (token.source, "not_required".to_string())
        }
        MultiChannelTransport::Whatsapp => {
            let token = resolve_lifecycle_secret(
                config,
                config.whatsapp_access_token.as_deref(),
                WHATSAPP_TOKEN_INTEGRATION_ID,
            );
            if token.credential_store_unreadable {
                reason_codes.push("credential_store_unreadable".to_string());
            }
            if token.value.is_none() {
                reason_codes.push("missing_whatsapp_access_token".to_string());
            }

            let phone_id = resolve_lifecycle_secret(
                config,
                config.whatsapp_phone_number_id.as_deref(),
                WHATSAPP_PHONE_NUMBER_ID_INTEGRATION_ID,
            );
            if phone_id.credential_store_unreadable {
                reason_codes.push("credential_store_unreadable".to_string());
            }
            if phone_id.value.is_none() {
                reason_codes.push("missing_whatsapp_phone_number_id".to_string());
            }

            (token.source, phone_id.source)
        }
    };

    if require_ingress_file {
        if !ingress_exists {
            reason_codes.push("ingress_missing".to_string());
        } else if !ingress_is_file {
            reason_codes.push("ingress_not_file".to_string());
        }
    } else if ingress_exists && !ingress_is_file {
        reason_codes.push("ingress_not_file".to_string());
    }

    let readiness_status = if reason_codes.is_empty() {
        "pass"
    } else {
        "fail"
    }
    .to_string();
    let reason_codes = if reason_codes.is_empty() {
        vec!["ready".to_string()]
    } else {
        reason_codes
    };

    ChannelReadiness {
        readiness_status,
        reason_codes,
        ingress_file,
        ingress_exists,
        ingress_is_file,
        token_source,
        phone_number_source,
    }
}

fn resolve_lifecycle_secret(
    config: &MultiChannelLifecycleCommandConfig,
    direct_secret: Option<&str>,
    integration_id: &str,
) -> ResolvedSecret {
    if let Some(secret) = resolve_non_empty_cli_value(direct_secret) {
        return ResolvedSecret {
            value: Some(secret),
            source: "cli_or_env".to_string(),
            credential_store_unreadable: false,
        };
    }
    if !config.credential_store_path.exists() {
        return ResolvedSecret {
            value: None,
            source: "missing".to_string(),
            credential_store_unreadable: false,
        };
    }

    let store = match load_credential_store(
        &config.credential_store_path,
        config.credential_store_encryption,
        config.credential_store_key.as_deref(),
    ) {
        Ok(store) => store,
        Err(_) => {
            return ResolvedSecret {
                value: None,
                source: "credential_store_error".to_string(),
                credential_store_unreadable: true,
            }
        }
    };
    let Some(record) = store.integrations.get(integration_id) else {
        return ResolvedSecret {
            value: None,
            source: "missing".to_string(),
            credential_store_unreadable: false,
        };
    };
    if record.revoked {
        return ResolvedSecret {
            value: None,
            source: "credential_store_revoked".to_string(),
            credential_store_unreadable: false,
        };
    }
    let value = record
        .secret
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let source = if value.is_some() {
        "credential_store".to_string()
    } else {
        "missing".to_string()
    };
    ResolvedSecret {
        value,
        source,
        credential_store_unreadable: false,
    }
}

fn render_multi_channel_lifecycle_report(report: &MultiChannelLifecycleReport) -> String {
    let reason_codes = if report.reason_codes.is_empty() {
        "none".to_string()
    } else {
        report.reason_codes.join(",")
    };
    format!(
        "multi-channel lifecycle: action={} channel={} lifecycle_status={} readiness_status={} reason_codes={} ingress_file={} ingress_exists={} ingress_is_file={} token_source={} phone_number_source={} state_path={} state_persisted={} updated_unix_ms={}",
        report.action,
        report.channel,
        report.lifecycle_status,
        report.readiness_status,
        reason_codes,
        report.ingress_file,
        report.ingress_exists,
        report.ingress_is_file,
        report.token_source,
        report.phone_number_source,
        report.state_path,
        report.state_persisted,
        report.updated_unix_ms
    )
}

#[cfg(test)]
mod tests {
    use super::{
        execute_multi_channel_lifecycle_action, lifecycle_state_path_for_dir,
        load_multi_channel_lifecycle_state, probe_channel_readiness,
        save_multi_channel_lifecycle_state, LifecycleActionKind, MultiChannelLifecycleChannelState,
        MultiChannelLifecycleCommandConfig, MultiChannelLifecycleStateFile,
    };
    use crate::credentials::{CredentialStoreData, IntegrationCredentialStoreRecord};
    use crate::multi_channel_contract::MultiChannelTransport;
    use crate::{save_credential_store, CredentialStoreEncryptionMode};
    use std::collections::BTreeMap;
    use std::path::Path;
    use tempfile::tempdir;

    fn test_config(root: &Path) -> MultiChannelLifecycleCommandConfig {
        MultiChannelLifecycleCommandConfig {
            state_dir: root.join(".tau/multi-channel"),
            ingress_dir: root.join(".tau/multi-channel/live-ingress"),
            credential_store_path: root.join(".tau/credentials.json"),
            credential_store_encryption: CredentialStoreEncryptionMode::None,
            credential_store_key: None,
            telegram_bot_token: None,
            discord_bot_token: None,
            whatsapp_access_token: None,
            whatsapp_phone_number_id: None,
        }
    }

    #[test]
    fn unit_probe_channel_readiness_reports_missing_prerequisites() {
        let temp = tempdir().expect("tempdir");
        let config = test_config(temp.path());
        let report = probe_channel_readiness(&config, MultiChannelTransport::Telegram, true);
        assert_eq!(report.readiness_status, "fail");
        assert!(report
            .reason_codes
            .contains(&"missing_telegram_bot_token".to_string()));
        assert!(report.reason_codes.contains(&"ingress_missing".to_string()));
    }

    #[test]
    fn functional_login_action_creates_ingress_and_persists_initialized_state() {
        let temp = tempdir().expect("tempdir");
        let mut config = test_config(temp.path());
        config.telegram_bot_token = Some("telegram-secret".to_string());

        let report = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Login,
            MultiChannelTransport::Telegram,
        )
        .expect("login should succeed");
        assert_eq!(report.lifecycle_status, "initialized");
        assert_eq!(report.readiness_status, "pass");
        assert!(report.ingress_exists);
        assert!(report.ingress_is_file);

        let state_path = lifecycle_state_path_for_dir(&config.state_dir);
        let state = load_multi_channel_lifecycle_state(&state_path).expect("state");
        let entry = state.channels.get("telegram").expect("telegram entry");
        assert_eq!(entry.lifecycle_status, "initialized");
        assert_eq!(entry.last_action, "login");
    }

    #[test]
    fn integration_login_status_logout_probe_flow_roundtrips_channel_state() {
        let temp = tempdir().expect("tempdir");
        let mut config = test_config(temp.path());
        config.discord_bot_token = Some("discord-secret".to_string());

        let login = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Login,
            MultiChannelTransport::Discord,
        )
        .expect("login");
        assert_eq!(login.lifecycle_status, "initialized");

        let status = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Status,
            MultiChannelTransport::Discord,
        )
        .expect("status");
        assert_eq!(status.lifecycle_status, "initialized");
        assert_eq!(status.readiness_status, "pass");

        let logout = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Logout,
            MultiChannelTransport::Discord,
        )
        .expect("logout");
        assert_eq!(logout.lifecycle_status, "logged_out");
        assert_eq!(logout.reason_codes, vec!["logout_requested".to_string()]);

        let probe = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Probe,
            MultiChannelTransport::Discord,
        )
        .expect("probe");
        assert_eq!(probe.lifecycle_status, "ready");
        assert_eq!(probe.readiness_status, "pass");
    }

    #[test]
    fn integration_login_action_resolves_store_backed_secret_when_cli_secret_missing() {
        let temp = tempdir().expect("tempdir");
        let config = test_config(temp.path());
        let mut integrations = BTreeMap::new();
        integrations.insert(
            "telegram-bot-token".to_string(),
            IntegrationCredentialStoreRecord {
                secret: Some("store-telegram-secret".to_string()),
                revoked: false,
                updated_unix: Some(1),
            },
        );
        save_credential_store(
            &config.credential_store_path,
            &CredentialStoreData {
                encryption: CredentialStoreEncryptionMode::None,
                providers: BTreeMap::new(),
                integrations,
            },
            config.credential_store_key.as_deref(),
        )
        .expect("save store");

        let report = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Login,
            MultiChannelTransport::Telegram,
        )
        .expect("login");
        assert_eq!(report.lifecycle_status, "initialized");
        assert_eq!(report.token_source, "credential_store");
    }

    #[test]
    fn regression_action_fails_on_corrupted_lifecycle_state_file() {
        let temp = tempdir().expect("tempdir");
        let config = test_config(temp.path());
        let state_path = lifecycle_state_path_for_dir(&config.state_dir);
        std::fs::create_dir_all(state_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&state_path, "{not-json").expect("write corrupted state");

        let error = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Status,
            MultiChannelTransport::Telegram,
        )
        .expect_err("corrupted state should fail");
        assert!(error
            .to_string()
            .contains("failed to parse multi-channel lifecycle state"));
    }

    #[test]
    fn regression_probe_whatsapp_reports_missing_phone_id_when_token_present() {
        let temp = tempdir().expect("tempdir");
        let mut config = test_config(temp.path());
        config.whatsapp_access_token = Some("wa-token".to_string());

        let report = execute_multi_channel_lifecycle_action(
            &config,
            LifecycleActionKind::Probe,
            MultiChannelTransport::Whatsapp,
        )
        .expect("probe");
        assert_eq!(report.lifecycle_status, "probe_failed");
        assert!(report
            .reason_codes
            .contains(&"missing_whatsapp_phone_number_id".to_string()));
    }

    #[test]
    fn regression_save_and_reload_state_roundtrips_schema_and_channel_rows() {
        let temp = tempdir().expect("tempdir");
        let config = test_config(temp.path());
        let state_path = lifecycle_state_path_for_dir(&config.state_dir);
        let mut channels = BTreeMap::new();
        channels.insert(
            "telegram".to_string(),
            MultiChannelLifecycleChannelState {
                lifecycle_status: "initialized".to_string(),
                reason_codes: vec!["ready".to_string()],
                last_action: "login".to_string(),
                last_updated_unix_ms: 10,
                last_login_unix_ms: 10,
                last_logout_unix_ms: 0,
                last_probe_unix_ms: 0,
            },
        );
        save_multi_channel_lifecycle_state(
            &state_path,
            &MultiChannelLifecycleStateFile {
                schema_version: 1,
                channels,
            },
        )
        .expect("save");
        let reloaded = load_multi_channel_lifecycle_state(&state_path).expect("reload");
        assert_eq!(reloaded.schema_version, 1);
        assert_eq!(
            reloaded
                .channels
                .get("telegram")
                .expect("channel")
                .lifecycle_status,
            "initialized"
        );
    }
}
