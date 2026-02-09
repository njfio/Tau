use super::*;

pub(crate) const RELEASE_CHANNEL_USAGE: &str =
    "usage: /release-channel [show|set <stable|beta|dev>]";
pub(crate) const RELEASE_CHANNEL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReleaseChannel {
    Stable,
    Beta,
    Dev,
}

impl ReleaseChannel {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ReleaseChannel::Stable => "stable",
            ReleaseChannel::Beta => "beta",
            ReleaseChannel::Dev => "dev",
        }
    }
}

impl std::fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ReleaseChannel {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "dev" => Ok(Self::Dev),
            _ => bail!(
                "invalid release channel '{}'; expected stable|beta|dev",
                value
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReleaseChannelCommand {
    Show,
    Set(ReleaseChannel),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ReleaseChannelStoreFile {
    pub(crate) schema_version: u32,
    pub(crate) release_channel: ReleaseChannel,
}

pub(crate) fn default_release_channel_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()
        .context("failed to resolve current working directory")?
        .join(".tau")
        .join("release-channel.json"))
}

pub(crate) fn parse_release_channel_command(command_args: &str) -> Result<ReleaseChannelCommand> {
    let tokens = command_args
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return Ok(ReleaseChannelCommand::Show);
    }

    if tokens.len() == 1 && tokens[0] == "show" {
        return Ok(ReleaseChannelCommand::Show);
    }

    if tokens.len() == 2 && tokens[0] == "set" {
        let channel = tokens[1].parse::<ReleaseChannel>()?;
        return Ok(ReleaseChannelCommand::Set(channel));
    }

    bail!("{RELEASE_CHANNEL_USAGE}");
}

pub(crate) fn load_release_channel_store(path: &Path) -> Result<Option<ReleaseChannel>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read release channel file {}", path.display()))?;
    let parsed = serde_json::from_str::<ReleaseChannelStoreFile>(&raw)
        .with_context(|| format!("failed to parse release channel file {}", path.display()))?;
    if parsed.schema_version != RELEASE_CHANNEL_SCHEMA_VERSION {
        bail!(
            "unsupported release channel schema_version {} in {} (expected {})",
            parsed.schema_version,
            path.display(),
            RELEASE_CHANNEL_SCHEMA_VERSION
        );
    }
    Ok(Some(parsed.release_channel))
}

pub(crate) fn save_release_channel_store(path: &Path, channel: ReleaseChannel) -> Result<()> {
    let payload = ReleaseChannelStoreFile {
        schema_version: RELEASE_CHANNEL_SCHEMA_VERSION,
        release_channel: channel,
    };
    let mut encoded =
        serde_json::to_string_pretty(&payload).context("failed to encode release channel store")?;
    encoded.push('\n');
    let parent = path.parent().ok_or_else(|| {
        anyhow!(
            "release channel path {} does not have a parent directory",
            path.display()
        )
    })?;
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create release channel directory {}",
            parent.display()
        )
    })?;
    write_text_atomic(path, &encoded)
}

pub(crate) fn execute_release_channel_command(command_args: &str, path: &Path) -> String {
    let command = match parse_release_channel_command(command_args) {
        Ok(command) => command,
        Err(error) => {
            return format!(
                "release channel error: path={} error={error}",
                path.display()
            );
        }
    };

    match command {
        ReleaseChannelCommand::Show => match load_release_channel_store(path) {
            Ok(Some(channel)) => format!(
                "release channel: path={} channel={} source=store",
                path.display(),
                channel
            ),
            Ok(None) => format!(
                "release channel: path={} channel={} source=default",
                path.display(),
                ReleaseChannel::Stable
            ),
            Err(error) => format!(
                "release channel error: path={} error={error}",
                path.display()
            ),
        },
        ReleaseChannelCommand::Set(channel) => match save_release_channel_store(path, channel) {
            Ok(()) => format!(
                "release channel set: path={} channel={} status=saved",
                path.display(),
                channel
            ),
            Err(error) => format!(
                "release channel error: path={} error={error}",
                path.display()
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_parse_release_channel_command_supports_show_and_set() {
        assert_eq!(
            parse_release_channel_command("").expect("default command"),
            ReleaseChannelCommand::Show
        );
        assert_eq!(
            parse_release_channel_command("show").expect("show command"),
            ReleaseChannelCommand::Show
        );
        assert_eq!(
            parse_release_channel_command("set beta").expect("set command"),
            ReleaseChannelCommand::Set(ReleaseChannel::Beta)
        );

        let invalid = parse_release_channel_command("set nightly").expect_err("invalid channel");
        assert!(invalid.to_string().contains("expected stable|beta|dev"));
    }

    #[test]
    fn functional_execute_release_channel_command_show_and_set_round_trip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("release-channel.json");

        let initial = execute_release_channel_command("", &path);
        assert!(initial.contains("channel=stable"));
        assert!(initial.contains("source=default"));

        let set_output = execute_release_channel_command("set dev", &path);
        assert!(set_output.contains("channel=dev"));
        assert!(set_output.contains("status=saved"));

        let show = execute_release_channel_command("show", &path);
        assert!(show.contains("channel=dev"));
        assert!(show.contains("source=store"));
    }

    #[test]
    fn integration_save_and_load_release_channel_store_round_trip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join(".tau/release-channel.json");
        save_release_channel_store(&path, ReleaseChannel::Beta).expect("save release channel");
        let loaded = load_release_channel_store(&path).expect("load release channel");
        assert_eq!(loaded, Some(ReleaseChannel::Beta));
    }

    #[test]
    fn regression_load_release_channel_store_rejects_invalid_schema_and_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("release-channel.json");
        std::fs::write(&path, r#"{"schema_version":99,"release_channel":"stable"}"#)
            .expect("write invalid schema");
        let schema_error = load_release_channel_store(&path).expect_err("schema should fail");
        assert!(schema_error
            .to_string()
            .contains("unsupported release channel schema_version"));

        std::fs::write(&path, "{invalid-json").expect("write malformed json");
        let parse_error = load_release_channel_store(&path).expect_err("parse should fail");
        assert!(parse_error
            .to_string()
            .contains("failed to parse release channel file"));
    }
}
