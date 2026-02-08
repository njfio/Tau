use std::{
    collections::HashSet,
    fs,
    hash::Hash,
    path::{Component, Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::Cli;

const EXTENSION_MANIFEST_SCHEMA_VERSION: u32 = 1;
const EXTENSION_TIMEOUT_MS_DEFAULT: u64 = 5_000;
const EXTENSION_TIMEOUT_MS_MAX: u64 = 300_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ExtensionManifestSummary {
    pub manifest_path: PathBuf,
    pub id: String,
    pub version: String,
    pub runtime: String,
    pub entrypoint: String,
    pub hook_count: usize,
    pub permission_count: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExtensionListEntry {
    pub manifest_path: PathBuf,
    pub id: String,
    pub version: String,
    pub runtime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExtensionListInvalidEntry {
    pub manifest_path: PathBuf,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExtensionListReport {
    pub list_root: PathBuf,
    pub entries: Vec<ExtensionListEntry>,
    pub invalid_entries: Vec<ExtensionListInvalidEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtensionManifest {
    schema_version: u32,
    id: String,
    version: String,
    runtime: ExtensionRuntime,
    entrypoint: String,
    #[serde(default)]
    hooks: Vec<ExtensionHook>,
    #[serde(default)]
    permissions: Vec<ExtensionPermission>,
    #[serde(default = "default_extension_timeout_ms")]
    timeout_ms: u64,
}

fn default_extension_timeout_ms() -> u64 {
    EXTENSION_TIMEOUT_MS_DEFAULT
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum ExtensionRuntime {
    Process,
    Wasm,
}

impl ExtensionRuntime {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::Wasm => "wasm",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
enum ExtensionHook {
    RunStart,
    RunEnd,
    PreToolCall,
    PostToolCall,
    MessageTransform,
    PolicyOverride,
}

impl ExtensionHook {
    fn as_str(&self) -> &'static str {
        match self {
            Self::RunStart => "run-start",
            Self::RunEnd => "run-end",
            Self::PreToolCall => "pre-tool-call",
            Self::PostToolCall => "post-tool-call",
            Self::MessageTransform => "message-transform",
            Self::PolicyOverride => "policy-override",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
enum ExtensionPermission {
    ReadFiles,
    WriteFiles,
    RunCommands,
    Network,
}

impl ExtensionPermission {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadFiles => "read-files",
            Self::WriteFiles => "write-files",
            Self::RunCommands => "run-commands",
            Self::Network => "network",
        }
    }
}

pub(crate) fn execute_extension_list_command(cli: &Cli) -> Result<()> {
    if !cli.extension_list {
        return Ok(());
    }
    let report = list_extension_manifests(&cli.extension_list_root)?;
    println!("{}", render_extension_list_report(&report));
    Ok(())
}

pub(crate) fn execute_extension_show_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.extension_show.as_ref() else {
        return Ok(());
    };
    let (manifest, summary) = load_and_validate_extension_manifest(path)?;
    println!("{}", render_extension_manifest_report(&summary, &manifest));
    Ok(())
}

pub(crate) fn execute_extension_validate_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.extension_validate.as_ref() else {
        return Ok(());
    };
    let summary = validate_extension_manifest(path)?;
    println!(
        "extension validate: path={} id={} version={} runtime={} entrypoint={} hooks={} permissions={} timeout_ms={}",
        summary.manifest_path.display(),
        summary.id,
        summary.version,
        summary.runtime,
        summary.entrypoint,
        summary.hook_count,
        summary.permission_count,
        summary.timeout_ms
    );
    Ok(())
}

pub(crate) fn validate_extension_manifest(path: &Path) -> Result<ExtensionManifestSummary> {
    let (_, summary) = load_and_validate_extension_manifest(path)?;
    Ok(summary)
}

fn load_and_validate_extension_manifest(
    path: &Path,
) -> Result<(ExtensionManifest, ExtensionManifestSummary)> {
    let manifest = load_extension_manifest(path)?;
    validate_manifest_schema(&manifest)?;
    validate_manifest_identifiers(&manifest)?;
    validate_entrypoint_path(&manifest.entrypoint)?;
    validate_unique(&manifest.hooks, "hooks")?;
    validate_unique(&manifest.permissions, "permissions")?;
    validate_timeout_ms(manifest.timeout_ms)?;
    let summary = ExtensionManifestSummary {
        manifest_path: path.to_path_buf(),
        id: manifest.id.clone(),
        version: manifest.version.clone(),
        runtime: manifest.runtime.as_str().to_string(),
        entrypoint: manifest.entrypoint.clone(),
        hook_count: manifest.hooks.len(),
        permission_count: manifest.permissions.len(),
        timeout_ms: manifest.timeout_ms,
    };
    Ok((manifest, summary))
}

fn render_extension_manifest_report(
    summary: &ExtensionManifestSummary,
    manifest: &ExtensionManifest,
) -> String {
    let mut hooks = manifest
        .hooks
        .iter()
        .map(|hook| hook.as_str().to_string())
        .collect::<Vec<_>>();
    hooks.sort();

    let mut permissions = manifest
        .permissions
        .iter()
        .map(|permission| permission.as_str().to_string())
        .collect::<Vec<_>>();
    permissions.sort();

    let hook_lines = if hooks.is_empty() {
        "- none".to_string()
    } else {
        hooks
            .iter()
            .map(|hook| format!("- {hook}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let permission_lines = if permissions.is_empty() {
        "- none".to_string()
    } else {
        permissions
            .iter()
            .map(|permission| format!("- {permission}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "extension show:\n- path: {}\n- id: {}\n- version: {}\n- runtime: {}\n- entrypoint: {}\n- timeout_ms: {}\n- hooks ({}):\n{}\n- permissions ({}):\n{}",
        summary.manifest_path.display(),
        summary.id,
        summary.version,
        summary.runtime,
        summary.entrypoint,
        summary.timeout_ms,
        summary.hook_count,
        hook_lines,
        summary.permission_count,
        permission_lines
    )
}

fn list_extension_manifests(root: &Path) -> Result<ExtensionListReport> {
    if !root.exists() {
        return Ok(ExtensionListReport {
            list_root: root.to_path_buf(),
            entries: vec![],
            invalid_entries: vec![],
        });
    }
    if !root.is_dir() {
        bail!(
            "extension list root '{}' is not a directory",
            root.display()
        );
    }

    let mut entries = Vec::new();
    let mut invalid_entries = Vec::new();
    for manifest_path in discover_manifest_paths(root)? {
        match validate_extension_manifest(&manifest_path) {
            Ok(summary) => entries.push(ExtensionListEntry {
                manifest_path: summary.manifest_path,
                id: summary.id,
                version: summary.version,
                runtime: summary.runtime,
            }),
            Err(error) => invalid_entries.push(ExtensionListInvalidEntry {
                manifest_path,
                error: error.to_string(),
            }),
        }
    }
    entries.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.version.cmp(&right.version))
            .then_with(|| left.manifest_path.cmp(&right.manifest_path))
    });
    invalid_entries.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));

    Ok(ExtensionListReport {
        list_root: root.to_path_buf(),
        entries,
        invalid_entries,
    })
}

fn discover_manifest_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let read_dir = fs::read_dir(root)
        .with_context(|| format!("failed to read extension root {}", root.display()))?;
    for entry in read_dir {
        let entry = entry
            .with_context(|| format!("failed to inspect extension root {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("extension.json");
            if manifest_path.is_file() {
                paths.push(manifest_path);
            }
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("json")
            && path.is_file()
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn render_extension_list_report(report: &ExtensionListReport) -> String {
    let mut lines = vec![format!(
        "extension list: root={} count={} invalid={}",
        report.list_root.display(),
        report.entries.len(),
        report.invalid_entries.len()
    )];
    for entry in &report.entries {
        lines.push(format!(
            "extension: id={} version={} runtime={} manifest={}",
            entry.id,
            entry.version,
            entry.runtime,
            entry.manifest_path.display()
        ));
    }
    for invalid in &report.invalid_entries {
        lines.push(format!(
            "invalid: manifest={} error={}",
            invalid.manifest_path.display(),
            invalid.error
        ));
    }
    lines.join("\n")
}

fn load_extension_manifest(path: &Path) -> Result<ExtensionManifest> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read extension manifest {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse extension manifest {}", path.display()))
}

fn validate_manifest_schema(manifest: &ExtensionManifest) -> Result<()> {
    if manifest.schema_version != EXTENSION_MANIFEST_SCHEMA_VERSION {
        bail!(
            "unsupported extension manifest schema '{}': expected {}",
            manifest.schema_version,
            EXTENSION_MANIFEST_SCHEMA_VERSION
        );
    }
    Ok(())
}

fn validate_manifest_identifiers(manifest: &ExtensionManifest) -> Result<()> {
    validate_non_empty_field("id", &manifest.id)?;
    validate_non_empty_field("version", &manifest.version)?;
    Ok(())
}

fn validate_non_empty_field(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("extension manifest '{}' must not be empty", name);
    }
    Ok(())
}

fn validate_entrypoint_path(entrypoint: &str) -> Result<()> {
    let trimmed = entrypoint.trim();
    if trimmed.is_empty() {
        bail!("extension manifest 'entrypoint' must not be empty");
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        bail!(
            "extension manifest entrypoint '{}' must be relative",
            trimmed
        );
    }
    for component in path.components() {
        match component {
            Component::ParentDir => {
                bail!(
                    "extension manifest entrypoint '{}' must not contain parent traversals",
                    trimmed
                );
            }
            Component::Prefix(_) | Component::RootDir => {
                bail!(
                    "extension manifest entrypoint '{}' must be relative",
                    trimmed
                );
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(())
}

fn validate_unique<T>(entries: &[T], field_name: &str) -> Result<()>
where
    T: Eq + Hash,
{
    let mut seen = HashSet::new();
    for entry in entries {
        if !seen.insert(entry) {
            bail!(
                "extension manifest '{}' contains duplicate entries",
                field_name
            );
        }
    }
    Ok(())
}

fn validate_timeout_ms(timeout_ms: u64) -> Result<()> {
    if timeout_ms == 0 {
        bail!("extension manifest 'timeout_ms' must be greater than 0");
    }
    if timeout_ms > EXTENSION_TIMEOUT_MS_MAX {
        bail!(
            "extension manifest 'timeout_ms' must be <= {}",
            EXTENSION_TIMEOUT_MS_MAX
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        list_extension_manifests, render_extension_list_report, render_extension_manifest_report,
        validate_extension_manifest, ExtensionHook, ExtensionListReport, ExtensionManifest,
        ExtensionManifestSummary, ExtensionPermission, ExtensionRuntime,
    };
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn unit_validate_extension_manifest_accepts_minimal_schema() {
        let temp = tempdir().expect("tempdir");
        let manifest_path = temp.path().join("extension.json");
        std::fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "id": "issue-assistant",
  "version": "0.1.0",
  "runtime": "process",
  "entrypoint": "bin/assistant"
}"#,
        )
        .expect("write manifest");

        let summary = validate_extension_manifest(&manifest_path).expect("valid manifest");
        assert_eq!(summary.id, "issue-assistant");
        assert_eq!(summary.version, "0.1.0");
        assert_eq!(summary.runtime, "process");
        assert_eq!(summary.entrypoint, "bin/assistant");
        assert_eq!(summary.hook_count, 0);
        assert_eq!(summary.permission_count, 0);
        assert_eq!(summary.timeout_ms, 5_000);
    }

    #[test]
    fn regression_validate_extension_manifest_rejects_parent_dir_entrypoint() {
        let temp = tempdir().expect("tempdir");
        let manifest_path = temp.path().join("extension.json");
        std::fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "id": "issue-assistant",
  "version": "0.1.0",
  "runtime": "process",
  "entrypoint": "../escape.sh"
}"#,
        )
        .expect("write manifest");

        let error =
            validate_extension_manifest(&manifest_path).expect_err("parent traversal should fail");
        assert!(error
            .to_string()
            .contains("must not contain parent traversals"));
    }

    #[test]
    fn regression_validate_extension_manifest_rejects_duplicate_hooks() {
        let temp = tempdir().expect("tempdir");
        let manifest_path = temp.path().join("extension.json");
        std::fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "id": "issue-assistant",
  "version": "0.1.0",
  "runtime": "process",
  "entrypoint": "bin/assistant",
  "hooks": ["run-start", "run-start"]
}"#,
        )
        .expect("write manifest");

        let error =
            validate_extension_manifest(&manifest_path).expect_err("duplicate hooks should fail");
        assert!(error.to_string().contains("contains duplicate entries"));
    }

    #[test]
    fn unit_render_extension_manifest_report_is_deterministic() {
        let summary = ExtensionManifestSummary {
            manifest_path: PathBuf::from("extensions/issue-assistant/extension.json"),
            id: "issue-assistant".to_string(),
            version: "0.1.0".to_string(),
            runtime: "process".to_string(),
            entrypoint: "bin/assistant".to_string(),
            hook_count: 2,
            permission_count: 2,
            timeout_ms: 60_000,
        };
        let manifest = ExtensionManifest {
            schema_version: 1,
            id: "issue-assistant".to_string(),
            version: "0.1.0".to_string(),
            runtime: ExtensionRuntime::Process,
            entrypoint: "bin/assistant".to_string(),
            hooks: vec![ExtensionHook::RunStart, ExtensionHook::RunEnd],
            permissions: vec![ExtensionPermission::Network, ExtensionPermission::ReadFiles],
            timeout_ms: 60_000,
        };

        let report = render_extension_manifest_report(&summary, &manifest);
        assert!(report.contains("extension show:"));
        assert!(report.contains("- id: issue-assistant"));
        assert!(report.contains("- hooks (2):\n- run-end\n- run-start"));
        assert!(report.contains("- permissions (2):\n- network\n- read-files"));
    }

    #[test]
    fn unit_render_extension_list_report_is_deterministic() {
        let report = ExtensionListReport {
            list_root: PathBuf::from("extensions"),
            entries: vec![super::ExtensionListEntry {
                manifest_path: PathBuf::from("extensions/issue-assistant/extension.json"),
                id: "issue-assistant".to_string(),
                version: "0.1.0".to_string(),
                runtime: "process".to_string(),
            }],
            invalid_entries: vec![super::ExtensionListInvalidEntry {
                manifest_path: PathBuf::from("extensions/bad/extension.json"),
                error: "unsupported extension manifest schema".to_string(),
            }],
        };

        let rendered = render_extension_list_report(&report);
        assert!(rendered.contains("extension list: root=extensions count=1 invalid=1"));
        assert!(rendered.contains(
            "extension: id=issue-assistant version=0.1.0 runtime=process manifest=extensions/issue-assistant/extension.json"
        ));
        assert!(rendered.contains("invalid: manifest=extensions/bad/extension.json error=unsupported extension manifest schema"));
    }

    #[test]
    fn regression_list_extension_manifests_reports_invalid_entries_without_failing() {
        let temp = tempdir().expect("tempdir");
        let good_dir = temp.path().join("good");
        fs::create_dir_all(&good_dir).expect("create good dir");
        fs::write(
            good_dir.join("extension.json"),
            r#"{
  "schema_version": 1,
  "id": "issue-assistant",
  "version": "0.1.0",
  "runtime": "process",
  "entrypoint": "bin/assistant"
}"#,
        )
        .expect("write valid extension");

        let bad_dir = temp.path().join("bad");
        fs::create_dir_all(&bad_dir).expect("create bad dir");
        fs::write(
            bad_dir.join("extension.json"),
            r#"{
  "schema_version": 9,
  "id": "broken",
  "version": "0.1.0",
  "runtime": "process",
  "entrypoint": "bin/assistant"
}"#,
        )
        .expect("write invalid extension");

        let report = list_extension_manifests(temp.path()).expect("list should succeed");
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.invalid_entries.len(), 1);
        assert_eq!(report.entries[0].id, "issue-assistant");
        assert!(report.invalid_entries[0]
            .error
            .contains("unsupported extension manifest schema"));
    }

    #[test]
    fn regression_list_extension_manifests_rejects_non_directory_root() {
        let temp = tempdir().expect("tempdir");
        let root_file = temp.path().join("extensions.json");
        fs::write(&root_file, "{}").expect("write root file");

        let error =
            list_extension_manifests(&root_file).expect_err("non-directory root should fail");
        assert!(error.to_string().contains("is not a directory"));
    }
}
