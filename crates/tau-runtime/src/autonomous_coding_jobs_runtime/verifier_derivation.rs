use std::{path::Path, process::Command};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RepoAwareVerifierDerivation {
    pub commands: Vec<String>,
    pub missing_inputs: Vec<String>,
}

pub(super) fn derive_concrete_docs_verifier_commands(title: &str, body: &str) -> Vec<String> {
    let combined = format!("{title}\n{body}");
    let normalized = combined.to_ascii_lowercase();
    if !contains_any(&normalized, &["readme", "docs", "documentation", "guide"]) {
        return Vec::new();
    }

    let Some(token) = extract_safe_docs_verifier_token(&combined) else {
        return Vec::new();
    };

    let grep_command = if normalized.contains("readme") {
        format!("grep -n {token} README.md")
    } else {
        format!("grep -R -n {token} docs")
    };
    vec!["git diff --check".to_string(), grep_command]
}

pub(super) fn derive_concrete_verifier_commands(
    repo_path: &Path,
    title: &str,
    body: &str,
) -> Vec<String> {
    let docs_commands = derive_concrete_docs_verifier_commands(title, body);
    if !docs_commands.is_empty() {
        return docs_commands;
    }
    derive_repo_aware_code_verifier_commands(repo_path, title, body)
}

pub(super) fn derive_repo_aware_code_verifier_commands(
    repo_path: &Path,
    title: &str,
    body: &str,
) -> Vec<String> {
    derive_repo_aware_code_verifier_plan(repo_path, title, body).commands
}

pub(super) fn derive_repo_aware_code_verifier_plan(
    repo_path: &Path,
    title: &str,
    body: &str,
) -> RepoAwareVerifierDerivation {
    let combined = format!("{title}\n{body}");
    let normalized = combined.to_ascii_lowercase();
    if !contains_any(
        &normalized,
        &[
            "cli",
            "command",
            "flag",
            "argument",
            "subcommand",
            "test",
            "panic",
            "rust",
            "crate",
            "compile",
            "clippy",
        ],
    ) {
        return RepoAwareVerifierDerivation::default();
    }

    let package_names = repo_cargo_package_names(repo_path);
    let package_name = resolve_referenced_package_name(&package_names, &combined);
    let test_filter = resolve_referenced_test_filter(&combined, &package_names);
    let mut missing_inputs = Vec::new();
    if package_name.is_none() {
        missing_inputs.push(missing_package_input(&package_names));
    }
    if test_filter.is_none() {
        missing_inputs.push(
            "exact quoted/backticked safe test filter token containing `spec`, `test`, `::`, or starting with `regression_`"
                .to_string(),
        );
    }

    let commands = match (package_name, test_filter) {
        (Some(package_name), Some(test_filter)) if is_safe_cargo_package_name(&package_name) => {
            vec![format!("cargo test -p {package_name} {test_filter}")]
        }
        _ => Vec::new(),
    };
    RepoAwareVerifierDerivation {
        commands,
        missing_inputs,
    }
}

fn repo_cargo_package_names(repo_path: &Path) -> Vec<String> {
    let Ok(output) = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo_path)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&output.stdout) else {
        return Vec::new();
    };
    let mut names = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|package| package.get("name"))
        .filter_map(serde_json::Value::as_str)
        .filter(|name| is_safe_cargo_package_name(name))
        .map(str::to_string)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

fn resolve_referenced_package_name(package_names: &[String], text: &str) -> Option<String> {
    let normalized = text.to_ascii_lowercase();
    let mut candidates = package_names.to_vec();
    candidates.sort_by_key(|name| std::cmp::Reverse(name.len()));
    candidates
        .into_iter()
        .find(|name| text_contains_safe_token(&normalized, &name.to_ascii_lowercase()))
}

fn resolve_referenced_test_filter(text: &str, package_names: &[String]) -> Option<String> {
    extract_safe_quoted_tokens(text).into_iter().find(|token| {
        is_safe_test_filter_token(token)
            && !package_names
                .iter()
                .any(|package| package.eq_ignore_ascii_case(token))
    })
}

fn text_contains_safe_token(text: &str, token: &str) -> bool {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':')))
        .any(|part| part == token)
}

fn extract_safe_docs_verifier_token(text: &str) -> Option<String> {
    extract_safe_quoted_tokens(text)
        .into_iter()
        .find(|token| is_safe_docs_verifier_token(token))
}

fn extract_safe_quoted_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for delimiter in ['`', '"'] {
        let mut parts = text.split(delimiter);
        while let Some(_) = parts.next() {
            let Some(candidate) = parts.next() else {
                break;
            };
            let token = candidate.trim();
            if is_safe_docs_verifier_token(token) {
                tokens.push(token.to_string());
            }
        }
    }
    tokens
}

fn is_safe_docs_verifier_token(token: &str) -> bool {
    let len = token.len();
    (3..=96).contains(&len)
        && !token.starts_with('-')
        && token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))
}

fn is_safe_cargo_package_name(package_name: &str) -> bool {
    let len = package_name.len();
    (1..=96).contains(&len)
        && !package_name.starts_with('-')
        && package_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn is_safe_test_filter_token(token: &str) -> bool {
    is_safe_docs_verifier_token(token)
        && (token.contains("spec")
            || token.contains("test")
            || token.contains("::")
            || token.starts_with("regression_"))
}

fn missing_package_input(package_names: &[String]) -> String {
    if package_names.is_empty() {
        return "Cargo workspace package metadata from `cargo metadata`".to_string();
    }
    format!(
        "actual Cargo package name present in this repository (available: {})",
        package_sample(package_names)
    )
}

fn package_sample(package_names: &[String]) -> String {
    const MAX_PACKAGE_SAMPLE: usize = 5;
    let mut sample = package_names
        .iter()
        .take(MAX_PACKAGE_SAMPLE)
        .cloned()
        .collect::<Vec<_>>();
    if package_names.len() > MAX_PACKAGE_SAMPLE {
        sample.push("...".to_string());
    }
    sample.join(", ")
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
