use std::{
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::Cli;

const PACKAGE_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PackageManifestSummary {
    pub manifest_path: PathBuf,
    pub name: String,
    pub version: String,
    pub template_count: usize,
    pub skill_count: usize,
    pub extension_count: usize,
    pub theme_count: usize,
    pub total_components: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageInstallReport {
    pub manifest_path: PathBuf,
    pub install_root: PathBuf,
    pub package_dir: PathBuf,
    pub name: String,
    pub version: String,
    pub manifest_status: FileUpsertOutcome,
    pub installed: usize,
    pub updated: usize,
    pub skipped: usize,
    pub total_components: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileUpsertOutcome {
    Installed,
    Updated,
    Skipped,
}

impl FileUpsertOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Updated => "updated",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageManifest {
    schema_version: u32,
    name: String,
    version: String,
    #[serde(default)]
    templates: Vec<PackageComponent>,
    #[serde(default)]
    skills: Vec<PackageComponent>,
    #[serde(default)]
    extensions: Vec<PackageComponent>,
    #[serde(default)]
    themes: Vec<PackageComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageComponent {
    id: String,
    path: String,
}

pub(crate) fn execute_package_validate_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.package_validate.as_ref() else {
        return Ok(());
    };
    let summary = validate_package_manifest(path)?;
    println!(
        "package validate: path={} name={} version={} templates={} skills={} extensions={} themes={} total_components={}",
        summary.manifest_path.display(),
        summary.name,
        summary.version,
        summary.template_count,
        summary.skill_count,
        summary.extension_count,
        summary.theme_count,
        summary.total_components,
    );
    Ok(())
}

pub(crate) fn execute_package_show_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.package_show.as_ref() else {
        return Ok(());
    };
    let (manifest, summary) = load_and_validate_manifest(path)?;
    println!("{}", render_package_manifest_report(&summary, &manifest));
    Ok(())
}

pub(crate) fn execute_package_install_command(cli: &Cli) -> Result<()> {
    let Some(path) = cli.package_install.as_ref() else {
        return Ok(());
    };
    let report = install_package_manifest(path, &cli.package_install_root)?;
    println!("{}", render_package_install_report(&report));
    Ok(())
}

pub(crate) fn validate_package_manifest(path: &Path) -> Result<PackageManifestSummary> {
    let (_, summary) = load_and_validate_manifest(path)?;
    Ok(summary)
}

fn load_and_validate_manifest(path: &Path) -> Result<(PackageManifest, PackageManifestSummary)> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read package manifest {}", path.display()))?;
    let manifest = serde_json::from_str::<PackageManifest>(&raw)
        .with_context(|| format!("failed to parse package manifest JSON {}", path.display()))?;
    if manifest.schema_version != PACKAGE_MANIFEST_SCHEMA_VERSION {
        bail!(
            "unsupported package manifest schema: expected {}, found {}",
            PACKAGE_MANIFEST_SCHEMA_VERSION,
            manifest.schema_version
        );
    }
    let name = manifest.name.trim();
    if name.is_empty() {
        bail!("package manifest name must be non-empty");
    }
    if !is_semver_like(manifest.version.trim()) {
        bail!(
            "package manifest version '{}' must follow x.y.z numeric semver form",
            manifest.version
        );
    }

    let mut total_components = 0_usize;
    validate_component_set("templates", &manifest.templates)?;
    total_components = total_components.saturating_add(manifest.templates.len());
    validate_component_set("skills", &manifest.skills)?;
    total_components = total_components.saturating_add(manifest.skills.len());
    validate_component_set("extensions", &manifest.extensions)?;
    total_components = total_components.saturating_add(manifest.extensions.len());
    validate_component_set("themes", &manifest.themes)?;
    total_components = total_components.saturating_add(manifest.themes.len());
    if total_components == 0 {
        bail!("package manifest must declare at least one component");
    }

    let summary = PackageManifestSummary {
        manifest_path: path.to_path_buf(),
        name: name.to_string(),
        version: manifest.version.trim().to_string(),
        template_count: manifest.templates.len(),
        skill_count: manifest.skills.len(),
        extension_count: manifest.extensions.len(),
        theme_count: manifest.themes.len(),
        total_components,
    };
    Ok((manifest, summary))
}

fn install_package_manifest(
    manifest_path: &Path,
    install_root: &Path,
) -> Result<PackageInstallReport> {
    let (manifest, summary) = load_and_validate_manifest(manifest_path)?;
    let manifest_dir = manifest_path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let canonical_manifest_dir = std::fs::canonicalize(manifest_dir).with_context(|| {
        format!(
            "failed to canonicalize package manifest directory {}",
            manifest_dir.display()
        )
    })?;

    std::fs::create_dir_all(install_root)
        .with_context(|| format!("failed to create {}", install_root.display()))?;
    let package_dir = install_root
        .join(summary.name.as_str())
        .join(summary.version.as_str());
    std::fs::create_dir_all(&package_dir)
        .with_context(|| format!("failed to create {}", package_dir.display()))?;

    let manifest_status =
        upsert_file_from_source(manifest_path, &package_dir.join("package.json"))?;
    let mut report = PackageInstallReport {
        manifest_path: manifest_path.to_path_buf(),
        install_root: install_root.to_path_buf(),
        package_dir: package_dir.clone(),
        name: summary.name,
        version: summary.version,
        manifest_status,
        installed: 0,
        updated: 0,
        skipped: 0,
        total_components: summary.total_components,
    };

    install_component_set(
        "templates",
        &manifest.templates,
        &canonical_manifest_dir,
        &package_dir,
        &mut report,
    )?;
    install_component_set(
        "skills",
        &manifest.skills,
        &canonical_manifest_dir,
        &package_dir,
        &mut report,
    )?;
    install_component_set(
        "extensions",
        &manifest.extensions,
        &canonical_manifest_dir,
        &package_dir,
        &mut report,
    )?;
    install_component_set(
        "themes",
        &manifest.themes,
        &canonical_manifest_dir,
        &package_dir,
        &mut report,
    )?;

    Ok(report)
}

fn render_package_install_report(report: &PackageInstallReport) -> String {
    format!(
        "package install: manifest={} root={} package_dir={} name={} version={} manifest_status={} installed={} updated={} skipped={} total_components={}",
        report.manifest_path.display(),
        report.install_root.display(),
        report.package_dir.display(),
        report.name,
        report.version,
        report.manifest_status.as_str(),
        report.installed,
        report.updated,
        report.skipped,
        report.total_components
    )
}

fn install_component_set(
    kind: &str,
    components: &[PackageComponent],
    canonical_manifest_dir: &Path,
    package_dir: &Path,
    report: &mut PackageInstallReport,
) -> Result<()> {
    for component in components {
        let id = component.id.trim();
        let relative_path = PathBuf::from_str(component.path.trim())
            .map_err(|_| anyhow!("failed to parse {} path '{}'", kind, component.path.trim()))?;
        let source_path =
            resolve_component_source_path(kind, id, &relative_path, canonical_manifest_dir)?;
        let destination = package_dir.join(&relative_path);
        match upsert_file_from_source(&source_path, &destination)? {
            FileUpsertOutcome::Installed => report.installed = report.installed.saturating_add(1),
            FileUpsertOutcome::Updated => report.updated = report.updated.saturating_add(1),
            FileUpsertOutcome::Skipped => report.skipped = report.skipped.saturating_add(1),
        }
    }
    Ok(())
}

fn resolve_component_source_path(
    kind: &str,
    id: &str,
    relative_path: &Path,
    canonical_manifest_dir: &Path,
) -> Result<PathBuf> {
    let joined = canonical_manifest_dir.join(relative_path);
    let canonical_source = std::fs::canonicalize(&joined).with_context(|| {
        format!(
            "package manifest {} entry '{}' source '{}' does not exist",
            kind,
            id,
            relative_path.display()
        )
    })?;
    if !canonical_source.starts_with(canonical_manifest_dir) {
        bail!(
            "package manifest {} entry '{}' source '{}' resolves outside package manifest directory",
            kind,
            id,
            relative_path.display()
        );
    }
    let metadata = std::fs::metadata(&canonical_source).with_context(|| {
        format!(
            "failed to read metadata for package manifest {} entry '{}' source '{}'",
            kind,
            id,
            canonical_source.display()
        )
    })?;
    if !metadata.is_file() {
        bail!(
            "package manifest {} entry '{}' source '{}' must be a file",
            kind,
            id,
            relative_path.display()
        );
    }
    Ok(canonical_source)
}

fn upsert_file_from_source(source: &Path, destination: &Path) -> Result<FileUpsertOutcome> {
    let source_content = std::fs::read(source)
        .with_context(|| format!("failed to read source file {}", source.display()))?;
    let destination_exists = destination.exists();
    if destination_exists {
        if destination.is_dir() {
            bail!("destination '{}' is a directory", destination.display());
        }
        let existing_content = std::fs::read(destination).with_context(|| {
            format!(
                "failed to read existing destination file {}",
                destination.display()
            )
        })?;
        if existing_content == source_content {
            return Ok(FileUpsertOutcome::Skipped);
        }
    }

    let parent_dir = destination
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent_dir)
        .with_context(|| format!("failed to create {}", parent_dir.display()))?;
    std::fs::write(destination, source_content)
        .with_context(|| format!("failed to write destination file {}", destination.display()))?;
    if destination_exists {
        Ok(FileUpsertOutcome::Updated)
    } else {
        Ok(FileUpsertOutcome::Installed)
    }
}

fn render_package_manifest_report(
    summary: &PackageManifestSummary,
    manifest: &PackageManifest,
) -> String {
    let mut lines = vec![format!(
        "package show: path={} name={} version={} schema_version={} total_components={}",
        summary.manifest_path.display(),
        summary.name,
        summary.version,
        PACKAGE_MANIFEST_SCHEMA_VERSION,
        summary.total_components
    )];
    append_component_section(&mut lines, "templates", &manifest.templates);
    append_component_section(&mut lines, "skills", &manifest.skills);
    append_component_section(&mut lines, "extensions", &manifest.extensions);
    append_component_section(&mut lines, "themes", &manifest.themes);
    lines.join("\n")
}

fn append_component_section(lines: &mut Vec<String>, label: &str, components: &[PackageComponent]) {
    lines.push(format!("{} ({}):", label, components.len()));
    if components.is_empty() {
        lines.push("none".to_string());
        return;
    }
    for component in components {
        lines.push(format!(
            "- {} => {}",
            component.id.trim(),
            component.path.trim()
        ));
    }
}

fn validate_component_set(kind: &str, components: &[PackageComponent]) -> Result<()> {
    let mut seen_ids = std::collections::BTreeSet::new();
    for component in components {
        let id = component.id.trim();
        if id.is_empty() {
            bail!("package manifest {} entry id must be non-empty", kind);
        }
        if !seen_ids.insert(id.to_string()) {
            bail!("duplicate {} id '{}'", kind, id);
        }
        validate_relative_component_path(kind, id, component.path.trim())?;
    }
    Ok(())
}

fn validate_relative_component_path(kind: &str, id: &str, raw_path: &str) -> Result<()> {
    if raw_path.is_empty() {
        bail!(
            "package manifest {} entry '{}' path must be non-empty",
            kind,
            id
        );
    }
    let path = PathBuf::from_str(raw_path)
        .map_err(|_| anyhow!("failed to parse {} path '{}'", kind, raw_path))?;
    if path.is_absolute() {
        bail!(
            "package manifest {} entry '{}' path '{}' must be relative",
            kind,
            id,
            raw_path
        );
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!(
            "package manifest {} entry '{}' path '{}' must not contain parent traversals",
            kind,
            id,
            raw_path
        );
    }
    Ok(())
}

fn is_semver_like(raw: &str) -> bool {
    let mut parts = raw.split('.');
    let major = parts.next();
    let minor = parts.next();
    let patch = parts.next();
    if parts.next().is_some() {
        return false;
    }
    [major, minor, patch].into_iter().all(|part| {
        part.map(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::{
        install_package_manifest, load_and_validate_manifest, render_package_install_report,
        render_package_manifest_report, validate_package_manifest, FileUpsertOutcome,
    };

    #[cfg(unix)]
    fn create_file_symlink(source: &Path, destination: &Path) {
        std::os::unix::fs::symlink(source, destination).expect("create symlink");
    }

    #[cfg(windows)]
    fn create_file_symlink(source: &Path, destination: &Path) {
        std::os::windows::fs::symlink_file(source, destination).expect("create symlink");
    }

    #[cfg(not(any(unix, windows)))]
    fn create_file_symlink(_source: &Path, _destination: &Path) {
        panic!("symlink test requires unix or windows target");
    }

    #[test]
    fn unit_validate_package_manifest_accepts_minimal_semver_shape() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("manifest.json");
        std::fs::write(
            &path,
            r#"{
  "schema_version": 1,
  "name": "starter",
  "version": "1.2.3",
  "templates": [{"id":"review","path":"templates/review.txt"}]
}"#,
        )
        .expect("write manifest");

        let summary = validate_package_manifest(&path).expect("validate manifest");
        assert_eq!(summary.name, "starter");
        assert_eq!(summary.version, "1.2.3");
        assert_eq!(summary.template_count, 1);
        assert_eq!(summary.total_components, 1);
    }

    #[test]
    fn functional_validate_package_manifest_counts_components_across_categories() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("manifest.json");
        std::fs::write(
            &path,
            r#"{
  "schema_version": 1,
  "name": "bundle",
  "version": "2.0.0",
  "templates": [{"id":"review","path":"templates/review.txt"}],
  "skills": [{"id":"checks","path":"skills/checks/SKILL.md"}],
  "extensions": [{"id":"hooks","path":"extensions/hooks.json"}],
  "themes": [{"id":"solarized","path":"themes/solarized.json"}]
}"#,
        )
        .expect("write manifest");

        let summary = validate_package_manifest(&path).expect("validate manifest");
        assert_eq!(summary.template_count, 1);
        assert_eq!(summary.skill_count, 1);
        assert_eq!(summary.extension_count, 1);
        assert_eq!(summary.theme_count, 1);
        assert_eq!(summary.total_components, 4);
    }

    #[test]
    fn regression_validate_package_manifest_rejects_duplicate_ids_and_unsafe_paths() {
        let temp = tempdir().expect("tempdir");
        let duplicate_path = temp.path().join("duplicate.json");
        std::fs::write(
            &duplicate_path,
            r#"{
  "schema_version": 1,
  "name": "bundle",
  "version": "1.0.0",
  "templates": [
    {"id":"review","path":"templates/review.txt"},
    {"id":"review","path":"templates/review-alt.txt"}
  ]
}"#,
        )
        .expect("write duplicate manifest");
        let duplicate_error =
            validate_package_manifest(&duplicate_path).expect_err("duplicate ids should fail");
        assert!(duplicate_error
            .to_string()
            .contains("duplicate templates id"));

        let traversal_path = temp.path().join("traversal.json");
        std::fs::write(
            &traversal_path,
            r#"{
  "schema_version": 1,
  "name": "bundle",
  "version": "1.0.0",
  "templates": [{"id":"review","path":"../escape.txt"}]
}"#,
        )
        .expect("write traversal manifest");
        let traversal_error =
            validate_package_manifest(&traversal_path).expect_err("unsafe path should fail");
        assert!(traversal_error
            .to_string()
            .contains("must not contain parent traversals"));
    }

    #[test]
    fn regression_validate_package_manifest_rejects_invalid_schema_or_version() {
        let temp = tempdir().expect("tempdir");
        let schema_path = temp.path().join("schema.json");
        std::fs::write(
            &schema_path,
            r#"{
  "schema_version": 9,
  "name": "bundle",
  "version": "1.0.0",
  "templates": [{"id":"review","path":"templates/review.txt"}]
}"#,
        )
        .expect("write schema manifest");
        let schema_error =
            validate_package_manifest(&schema_path).expect_err("schema mismatch should fail");
        assert!(schema_error
            .to_string()
            .contains("unsupported package manifest schema"));

        let version_path = temp.path().join("version.json");
        std::fs::write(
            &version_path,
            r#"{
  "schema_version": 1,
  "name": "bundle",
  "version": "1.0",
  "templates": [{"id":"review","path":"templates/review.txt"}]
}"#,
        )
        .expect("write version manifest");
        let version_error =
            validate_package_manifest(&version_path).expect_err("invalid version should fail");
        assert!(version_error.to_string().contains("must follow x.y.z"));
    }

    #[test]
    fn unit_render_package_manifest_report_includes_category_inventory() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("render.json");
        std::fs::write(
            &path,
            r#"{
  "schema_version": 1,
  "name": "bundle",
  "version": "1.0.0",
  "templates": [{"id":"review","path":"templates/review.txt"}],
  "skills": [{"id":"checks","path":"skills/checks/SKILL.md"}]
}"#,
        )
        .expect("write manifest");

        let (manifest, summary) = load_and_validate_manifest(&path).expect("load manifest");
        let report = render_package_manifest_report(&summary, &manifest);
        assert!(report.contains("package show:"));
        assert!(report.contains("templates (1):"));
        assert!(report.contains("- review => templates/review.txt"));
        assert!(report.contains("skills (1):"));
        assert!(report.contains("extensions (0):"));
        assert!(report.contains("themes (0):"));
    }

    #[test]
    fn functional_install_package_manifest_copies_components_into_versioned_layout() {
        let temp = tempdir().expect("tempdir");
        let package_root = temp.path().join("bundle");
        let templates_dir = package_root.join("templates");
        let skills_dir = package_root.join("skills/checks");
        std::fs::create_dir_all(&templates_dir).expect("create templates dir");
        std::fs::create_dir_all(&skills_dir).expect("create skills dir");
        std::fs::write(templates_dir.join("review.txt"), "template body")
            .expect("write template source");
        std::fs::write(skills_dir.join("SKILL.md"), "# checks").expect("write skill source");

        let manifest_path = package_root.join("package.json");
        std::fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "name": "starter",
  "version": "1.0.0",
  "templates": [{"id":"review","path":"templates/review.txt"}],
  "skills": [{"id":"checks","path":"skills/checks/SKILL.md"}]
}"#,
        )
        .expect("write manifest");

        let install_root = temp.path().join("installed");
        let first =
            install_package_manifest(&manifest_path, &install_root).expect("install package");
        assert_eq!(first.name, "starter");
        assert_eq!(first.version, "1.0.0");
        assert_eq!(first.total_components, 2);
        assert_eq!(first.installed, 2);
        assert_eq!(first.updated, 0);
        assert_eq!(first.skipped, 0);
        assert_eq!(first.manifest_status, FileUpsertOutcome::Installed);
        assert_eq!(
            std::fs::read_to_string(install_root.join("starter/1.0.0/templates/review.txt"))
                .expect("read installed template"),
            "template body"
        );
        assert_eq!(
            std::fs::read_to_string(install_root.join("starter/1.0.0/skills/checks/SKILL.md"))
                .expect("read installed skill"),
            "# checks"
        );

        let second =
            install_package_manifest(&manifest_path, &install_root).expect("reinstall package");
        assert_eq!(second.installed, 0);
        assert_eq!(second.updated, 0);
        assert_eq!(second.skipped, 2);
        assert_eq!(second.manifest_status, FileUpsertOutcome::Skipped);
    }

    #[test]
    fn regression_install_package_manifest_rejects_missing_component_source() {
        let temp = tempdir().expect("tempdir");
        let package_root = temp.path().join("bundle");
        std::fs::create_dir_all(package_root.join("templates")).expect("create templates dir");
        let manifest_path = package_root.join("package.json");
        std::fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "name": "starter",
  "version": "1.0.0",
  "templates": [{"id":"review","path":"templates/missing.txt"}]
}"#,
        )
        .expect("write manifest");

        let install_root = temp.path().join("installed");
        let error = install_package_manifest(&manifest_path, &install_root)
            .expect_err("missing source should fail");
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn regression_install_package_manifest_rejects_symlink_escape() {
        let temp = tempdir().expect("tempdir");
        let package_root = temp.path().join("bundle");
        let templates_dir = package_root.join("templates");
        std::fs::create_dir_all(&templates_dir).expect("create templates dir");

        let outside_dir = temp.path().join("outside");
        std::fs::create_dir_all(&outside_dir).expect("create outside dir");
        let outside_file = outside_dir.join("secret.txt");
        std::fs::write(&outside_file, "outside").expect("write outside file");
        create_file_symlink(&outside_file, &templates_dir.join("escape.txt"));

        let manifest_path = package_root.join("package.json");
        std::fs::write(
            &manifest_path,
            r#"{
  "schema_version": 1,
  "name": "starter",
  "version": "1.0.0",
  "templates": [{"id":"review","path":"templates/escape.txt"}]
}"#,
        )
        .expect("write manifest");

        let install_root = temp.path().join("installed");
        let error = install_package_manifest(&manifest_path, &install_root)
            .expect_err("symlink escape should fail");
        assert!(error
            .to_string()
            .contains("resolves outside package manifest directory"));
    }

    #[test]
    fn unit_render_package_install_report_includes_status_and_counts() {
        let report = super::PackageInstallReport {
            manifest_path: Path::new("/tmp/source/package.json").to_path_buf(),
            install_root: Path::new("/tmp/install").to_path_buf(),
            package_dir: Path::new("/tmp/install/starter/1.0.0").to_path_buf(),
            name: "starter".to_string(),
            version: "1.0.0".to_string(),
            manifest_status: FileUpsertOutcome::Updated,
            installed: 1,
            updated: 2,
            skipped: 3,
            total_components: 6,
        };
        let rendered = render_package_install_report(&report);
        assert!(rendered.contains("package install:"));
        assert!(rendered.contains("manifest_status=updated"));
        assert!(rendered.contains("installed=1"));
        assert!(rendered.contains("updated=2"));
        assert!(rendered.contains("skipped=3"));
        assert!(rendered.contains("total_components=6"));
    }
}
