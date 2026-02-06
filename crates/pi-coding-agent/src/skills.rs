use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use reqwest::Url;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub content: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkillInstallReport {
    pub installed: usize,
    pub updated: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSkillSource {
    pub url: String,
    pub sha256: Option<String>,
}

pub fn load_catalog(dir: &Path) -> Result<Vec<Skill>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !dir.is_dir() {
        bail!("skills path '{}' is not a directory", dir.display());
    }

    let mut skills = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry.with_context(|| format!("failed to read entry in {}", dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read skill file {}", path.display()))?;
        skills.push(Skill {
            name: stem.to_string(),
            content,
            path,
        });
    }

    skills.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(skills)
}

pub fn install_skills(sources: &[PathBuf], destination_dir: &Path) -> Result<SkillInstallReport> {
    if sources.is_empty() {
        return Ok(SkillInstallReport::default());
    }

    fs::create_dir_all(destination_dir)
        .with_context(|| format!("failed to create {}", destination_dir.display()))?;

    let mut report = SkillInstallReport::default();
    for source in sources {
        if source.extension().and_then(|ext| ext.to_str()) != Some("md") {
            bail!("skill source '{}' must be a .md file", source.display());
        }

        let file_name = source
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid skill source '{}'", source.display()))?;
        let destination = destination_dir.join(file_name);

        let content = fs::read_to_string(source)
            .with_context(|| format!("failed to read skill source {}", source.display()))?;
        upsert_skill_file(&destination, &content, &mut report)?;
    }

    Ok(report)
}

pub fn resolve_remote_skill_sources(
    urls: &[String],
    sha256_values: &[String],
) -> Result<Vec<RemoteSkillSource>> {
    if sha256_values.is_empty() {
        return Ok(urls
            .iter()
            .map(|url| RemoteSkillSource {
                url: url.clone(),
                sha256: None,
            })
            .collect());
    }

    if urls.len() != sha256_values.len() {
        bail!(
            "--install-skill-url count ({}) must match --install-skill-sha256 count ({})",
            urls.len(),
            sha256_values.len()
        );
    }

    Ok(urls
        .iter()
        .zip(sha256_values.iter())
        .map(|(url, sha256)| RemoteSkillSource {
            url: url.clone(),
            sha256: Some(sha256.clone()),
        })
        .collect())
}

pub async fn install_remote_skills(
    sources: &[RemoteSkillSource],
    destination_dir: &Path,
) -> Result<SkillInstallReport> {
    if sources.is_empty() {
        return Ok(SkillInstallReport::default());
    }

    fs::create_dir_all(destination_dir)
        .with_context(|| format!("failed to create {}", destination_dir.display()))?;

    let client = reqwest::Client::new();
    let mut report = SkillInstallReport::default();

    for (index, source) in sources.iter().enumerate() {
        let url = Url::parse(&source.url)
            .with_context(|| format!("invalid skill URL '{}'", source.url))?;
        if !matches!(url.scheme(), "http" | "https") {
            bail!("unsupported skill URL scheme '{}'", url.scheme());
        }

        let response = client
            .get(url.clone())
            .send()
            .await
            .with_context(|| format!("failed to fetch skill URL '{}'", source.url))?;
        if !response.status().is_success() {
            bail!(
                "failed to fetch skill URL '{}' with status {}",
                source.url,
                response.status()
            );
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("failed to read skill response '{}'", source.url))?;
        if let Some(expected_sha256) = &source.sha256 {
            let actual_sha256 = sha256_hex(&bytes);
            let expected_sha256 = normalize_sha256(expected_sha256);
            if actual_sha256 != expected_sha256 {
                bail!(
                    "sha256 mismatch for '{}': expected {}, got {}",
                    source.url,
                    expected_sha256,
                    actual_sha256
                );
            }
        }

        let file_name = remote_skill_file_name(&url, index);
        let destination = destination_dir.join(file_name);
        let content = String::from_utf8(bytes.to_vec())
            .with_context(|| format!("skill content from '{}' is not UTF-8", source.url))?;

        upsert_skill_file(&destination, &content, &mut report)?;
    }

    Ok(report)
}

pub fn resolve_selected_skills(catalog: &[Skill], selected: &[String]) -> Result<Vec<Skill>> {
    let mut resolved = Vec::new();
    for name in selected {
        let skill = catalog
            .iter()
            .find(|skill| skill.name == *name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown skill '{}'", name))?;
        resolved.push(skill);
    }

    Ok(resolved)
}

pub fn augment_system_prompt(base: &str, skills: &[Skill]) -> String {
    let mut prompt = base.trim_end().to_string();
    for skill in skills {
        if !prompt.is_empty() {
            prompt.push_str("\n\n");
        }

        prompt.push_str("# Skill: ");
        prompt.push_str(&skill.name);
        prompt.push('\n');
        prompt.push_str(skill.content.trim());
    }

    prompt
}

fn upsert_skill_file(
    destination: &Path,
    content: &str,
    report: &mut SkillInstallReport,
) -> Result<()> {
    if destination.exists() {
        let existing = fs::read_to_string(destination)
            .with_context(|| format!("failed to read installed skill {}", destination.display()))?;
        if existing == content {
            report.skipped += 1;
            return Ok(());
        }

        fs::write(destination, content.as_bytes())
            .with_context(|| format!("failed to update skill {}", destination.display()))?;
        report.updated += 1;
        return Ok(());
    }

    fs::write(destination, content.as_bytes())
        .with_context(|| format!("failed to install skill {}", destination.display()))?;
    report.installed += 1;
    Ok(())
}

fn remote_skill_file_name(url: &Url, index: usize) -> String {
    let base_name = url
        .path_segments()
        .and_then(|segments| {
            segments
                .filter(|segment| !segment.is_empty())
                .next_back()
                .map(|segment| segment.to_string())
        })
        .unwrap_or_else(|| format!("remote-skill-{}", index + 1));

    if base_name.ends_with(".md") {
        base_name
    } else {
        format!("{base_name}.md")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn normalize_sha256(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use httpmock::prelude::*;
    use sha2::{Digest, Sha256};
    use tempfile::tempdir;

    use super::{
        augment_system_prompt, install_remote_skills, install_skills, load_catalog,
        resolve_remote_skill_sources, resolve_selected_skills, RemoteSkillSource, Skill,
        SkillInstallReport,
    };

    #[test]
    fn unit_load_catalog_reads_markdown_files_only() {
        let temp = tempdir().expect("tempdir");
        std::fs::write(temp.path().join("a.md"), "A").expect("write a");
        std::fs::write(temp.path().join("b.txt"), "B").expect("write b");
        std::fs::write(temp.path().join("c.md"), "C").expect("write c");

        let catalog = load_catalog(temp.path()).expect("catalog");
        let names = catalog
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a", "c"]);
    }

    #[test]
    fn functional_augment_system_prompt_preserves_selected_skill_order() {
        let skills = vec![
            Skill {
                name: "first".to_string(),
                content: "one".to_string(),
                path: "first.md".into(),
            },
            Skill {
                name: "second".to_string(),
                content: "two".to_string(),
                path: "second.md".into(),
            },
        ];

        let prompt = augment_system_prompt("base", &skills);
        assert!(prompt.contains("# Skill: first\none"));
        assert!(prompt.contains("# Skill: second\ntwo"));
        assert!(prompt.find("first").expect("first") < prompt.find("second").expect("second"));
    }

    #[test]
    fn regression_resolve_selected_skills_errors_on_unknown_skill() {
        let catalog = vec![Skill {
            name: "known".to_string(),
            content: "x".to_string(),
            path: "known.md".into(),
        }];

        let error = resolve_selected_skills(&catalog, &["missing".to_string()])
            .expect_err("unknown skill should fail");
        assert!(error.to_string().contains("unknown skill 'missing'"));
    }

    #[test]
    fn integration_load_and_resolve_selected_skills_roundtrip() {
        let temp = tempdir().expect("tempdir");
        std::fs::write(temp.path().join("alpha.md"), "alpha body").expect("write alpha");
        std::fs::write(temp.path().join("beta.md"), "beta body").expect("write beta");

        let catalog = load_catalog(temp.path()).expect("catalog");
        let selected =
            resolve_selected_skills(&catalog, &["beta".to_string(), "alpha".to_string()])
                .expect("resolve");
        assert_eq!(
            selected
                .iter()
                .map(|skill| skill.name.as_str())
                .collect::<Vec<_>>(),
            vec!["beta", "alpha"]
        );
    }

    #[test]
    fn unit_install_skills_copies_new_skill_files() {
        let temp = tempdir().expect("tempdir");
        let source = temp.path().join("source.md");
        std::fs::write(&source, "source").expect("write source");
        let install_dir = temp.path().join("skills");

        let report = install_skills(&[source], &install_dir).expect("install");
        assert_eq!(
            report,
            SkillInstallReport {
                installed: 1,
                updated: 0,
                skipped: 0
            }
        );
        assert_eq!(
            std::fs::read_to_string(install_dir.join("source.md")).expect("read installed"),
            "source"
        );
    }

    #[test]
    fn regression_install_skills_skips_when_content_unchanged() {
        let temp = tempdir().expect("tempdir");
        let install_dir = temp.path().join("skills");
        std::fs::create_dir_all(&install_dir).expect("mkdir");
        std::fs::write(install_dir.join("stable.md"), "same").expect("write installed");

        let source = temp.path().join("stable.md");
        std::fs::write(&source, "same").expect("write source");

        let report = install_skills(&[source], &install_dir).expect("install");
        assert_eq!(
            report,
            SkillInstallReport {
                installed: 0,
                updated: 0,
                skipped: 1
            }
        );
    }

    #[test]
    fn integration_install_skills_updates_existing_content() {
        let temp = tempdir().expect("tempdir");
        let install_dir = temp.path().join("skills");
        std::fs::create_dir_all(&install_dir).expect("mkdir");
        std::fs::write(install_dir.join("evolve.md"), "v1").expect("write installed");

        let source = temp.path().join("evolve.md");
        std::fs::write(&source, "v2").expect("write source");

        let report = install_skills(&[PathBuf::from(&source)], &install_dir).expect("install");
        assert_eq!(
            report,
            SkillInstallReport {
                installed: 0,
                updated: 1,
                skipped: 0
            }
        );
        assert_eq!(
            std::fs::read_to_string(install_dir.join("evolve.md")).expect("read installed"),
            "v2"
        );
    }

    #[test]
    fn regression_resolve_remote_skill_sources_requires_matching_sha_count() {
        let error = resolve_remote_skill_sources(
            &["https://example.com/a.md".to_string()],
            &["abc".to_string(), "def".to_string()],
        )
        .expect_err("mismatched lengths should fail");
        assert!(error.to_string().contains("count"));
    }

    #[tokio::test]
    async fn functional_install_remote_skills_fetches_and_verifies_checksum() {
        let server = MockServer::start();
        let body = "remote skill body";
        let checksum = format!("{:x}", Sha256::digest(body.as_bytes()));

        let remote = server.mock(|when, then| {
            when.method(GET).path("/skills/review.md");
            then.status(200).body(body);
        });

        let temp = tempdir().expect("tempdir");
        let destination = temp.path().join("skills");
        let report = install_remote_skills(
            &[RemoteSkillSource {
                url: format!("{}/skills/review.md", server.base_url()),
                sha256: Some(checksum),
            }],
            &destination,
        )
        .await
        .expect("remote install should succeed");

        assert_eq!(
            report,
            SkillInstallReport {
                installed: 1,
                updated: 0,
                skipped: 0
            }
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("review.md")).expect("read installed"),
            body
        );
        remote.assert_hits(1);
    }

    #[tokio::test]
    async fn regression_install_remote_skills_fails_on_checksum_mismatch() {
        let server = MockServer::start();
        let remote = server.mock(|when, then| {
            when.method(GET).path("/skills/check.md");
            then.status(200).body("payload");
        });

        let temp = tempdir().expect("tempdir");
        let destination = temp.path().join("skills");
        let error = install_remote_skills(
            &[RemoteSkillSource {
                url: format!("{}/skills/check.md", server.base_url()),
                sha256: Some("deadbeef".to_string()),
            }],
            &destination,
        )
        .await
        .expect_err("checksum mismatch should fail");

        assert!(error.to_string().contains("sha256 mismatch"));
        remote.assert_hits(1);
    }

    #[tokio::test]
    async fn integration_install_remote_skills_updates_existing_file() {
        let server = MockServer::start();
        let remote = server.mock(|when, then| {
            when.method(GET).path("/skills/sync");
            then.status(200).body("v2");
        });

        let temp = tempdir().expect("tempdir");
        let destination = temp.path().join("skills");
        std::fs::create_dir_all(&destination).expect("mkdir");
        std::fs::write(destination.join("sync.md"), "v1").expect("write existing");

        let report = install_remote_skills(
            &[RemoteSkillSource {
                url: format!("{}/skills/sync", server.base_url()),
                sha256: None,
            }],
            &destination,
        )
        .await
        .expect("remote update should succeed");

        assert_eq!(
            report,
            SkillInstallReport {
                installed: 0,
                updated: 1,
                skipped: 0
            }
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("sync.md")).expect("read updated"),
            "v2"
        );
        remote.assert_hits(1);
    }
}
