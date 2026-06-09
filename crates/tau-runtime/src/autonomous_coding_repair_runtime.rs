//! Provider repair adapter for durable autonomous coding jobs.
//!
//! The runtime owns the durable loop, but provider execution stays behind a
//! command adapter so live provider clients do not force a crate dependency
//! cycle. The adapter receives a JSON context file through environment
//! variables and must print one JSON edit payload to stdout.

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tau_agent_core::{
    CodingMissionControlledEdit, CodingMissionState, CodingWorkspaceCommandStatus,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingProviderRepairPolicy {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_attempts: u32,
    #[serde(default)]
    pub command: Option<PathBuf>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

impl AutonomousCodingProviderRepairPolicy {
    pub fn is_configured(&self) -> bool {
        self.enabled && self.max_attempts > 0 && self.command.is_some()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomousCodingProviderRepairStatus {
    Applied,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingProviderRepairEvidence {
    pub status: AutonomousCodingProviderRepairStatus,
    pub attempt_index: u32,
    pub reason_code: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub command_argv: Vec<String>,
    pub context_path: PathBuf,
    #[serde(default)]
    pub stdout_path: Option<PathBuf>,
    #[serde(default)]
    pub stderr_path: Option<PathBuf>,
    #[serde(default)]
    pub exit_status: Option<i32>,
    #[serde(default)]
    pub output_sha256: Option<String>,
    #[serde(default)]
    pub edits_count: usize,
    #[serde(default)]
    pub edit_paths: Vec<String>,
    #[serde(default)]
    pub failed_verifier_count: usize,
    #[serde(default)]
    pub diff_context_bytes: usize,
    #[serde(default)]
    pub error_summary: Option<String>,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingProviderRepairRun {
    pub evidence: AutonomousCodingProviderRepairEvidence,
    #[serde(default)]
    pub edits: Vec<CodingMissionControlledEdit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingProviderRepairContext {
    pub schema_version: u32,
    pub job_id: String,
    pub mission_id: String,
    pub attempt_index: u32,
    pub goal: String,
    pub repo_path: PathBuf,
    #[serde(default)]
    pub issue_url: Option<String>,
    pub verifier_commands: Vec<String>,
    pub failed_verifiers: Vec<AutonomousCodingVerifierFailureContext>,
    pub changed_files: Vec<String>,
    pub git_diff: String,
    pub created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutonomousCodingVerifierFailureContext {
    pub argv: Vec<String>,
    #[serde(default)]
    pub exit_status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProviderRepairPayload {
    Single(ProviderRepairPayloadEntry),
    Multi {
        edits: Vec<ProviderRepairPayloadEntry>,
    },
    FileMap {
        files: BTreeMap<String, String>,
        #[serde(default)]
        reason_code: Option<String>,
    },
    UnifiedDiff {
        diff: String,
        #[serde(default)]
        reason_code: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct ProviderRepairPayloadEntry {
    relative_path: String,
    contents: String,
    #[serde(default)]
    reason_code: Option<String>,
}

pub fn run_autonomous_coding_provider_repair(
    policy: &AutonomousCodingProviderRepairPolicy,
    state: &CodingMissionState,
    job_id: &str,
    attempt_index: u32,
    artifact_dir: &Path,
    started_unix_ms: u64,
) -> Result<AutonomousCodingProviderRepairRun> {
    let context = build_provider_repair_context(state, job_id, attempt_index, started_unix_ms)?;
    std::fs::create_dir_all(artifact_dir)?;
    let context_path = artifact_dir.join(format!("provider-repair-context-{attempt_index}.json"));
    write_json_atomic(&context_path, &context)?;

    let Some(command) = policy.command.as_ref() else {
        return Ok(provider_repair_failure_with_artifacts(
            policy,
            attempt_index,
            context_path,
            Vec::new(),
            None,
            None,
            None,
            context.failed_verifiers.len(),
            context.git_diff.len(),
            "provider_repair_command_missing",
            "provider repair command is not configured".to_string(),
            started_unix_ms,
        ));
    };

    let stdout_path = artifact_dir.join(format!("provider-repair-{attempt_index}.stdout.log"));
    let stderr_path = artifact_dir.join(format!("provider-repair-{attempt_index}.stderr.log"));
    let command_argv = provider_repair_command_argv(command, &policy.args);
    let output = Command::new(command)
        .args(&policy.args)
        .env("TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT", &context_path)
        .env(
            "TAU_AUTONOMOUS_CODING_REPAIR_ATTEMPT",
            attempt_index.to_string(),
        )
        .env(
            "TAU_AUTONOMOUS_CODING_REPAIR_PROVIDER",
            policy.provider.clone().unwrap_or_default(),
        )
        .env(
            "TAU_AUTONOMOUS_CODING_REPAIR_MODEL",
            policy.model.clone().unwrap_or_default(),
        )
        .current_dir(&state.repo_path)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return Ok(provider_repair_failure_with_artifacts(
                policy,
                attempt_index,
                context_path,
                command_argv,
                None,
                None,
                None,
                context.failed_verifiers.len(),
                context.git_diff.len(),
                "provider_repair_spawn_failed",
                error.to_string(),
                started_unix_ms,
            ));
        }
    };

    std::fs::write(&stdout_path, &output.stdout)?;
    std::fs::write(&stderr_path, &output.stderr)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stdout_sha = sha256_hex(stdout.as_bytes());
    if !output.status.success() {
        return Ok(provider_repair_failure_with_artifacts(
            policy,
            attempt_index,
            context_path,
            command_argv,
            Some(stdout_path),
            Some(stderr_path),
            output.status.code(),
            context.failed_verifiers.len(),
            context.git_diff.len(),
            "provider_repair_command_failed",
            format!(
                "provider repair command exited with status {}",
                output.status.code().unwrap_or(-1)
            ),
            started_unix_ms,
        ));
    }

    match parse_provider_repair_edits(&stdout, &state.repo_path) {
        Ok(edits) => {
            let edit_paths = edits
                .iter()
                .map(|edit| edit.relative_path.display().to_string())
                .collect::<Vec<_>>();
            Ok(AutonomousCodingProviderRepairRun {
                evidence: AutonomousCodingProviderRepairEvidence {
                    status: AutonomousCodingProviderRepairStatus::Applied,
                    attempt_index,
                    reason_code: "provider_repair_edit_parsed".to_string(),
                    provider: policy.provider.clone(),
                    model: policy.model.clone(),
                    command_argv,
                    context_path,
                    stdout_path: Some(stdout_path),
                    stderr_path: Some(stderr_path),
                    exit_status: output.status.code(),
                    output_sha256: Some(stdout_sha),
                    edits_count: edits.len(),
                    edit_paths,
                    failed_verifier_count: context.failed_verifiers.len(),
                    diff_context_bytes: context.git_diff.len(),
                    error_summary: None,
                    created_unix_ms: started_unix_ms,
                },
                edits,
            })
        }
        Err(error) => Ok(AutonomousCodingProviderRepairRun {
            evidence: AutonomousCodingProviderRepairEvidence {
                status: AutonomousCodingProviderRepairStatus::Rejected,
                attempt_index,
                reason_code: "provider_repair_output_invalid".to_string(),
                provider: policy.provider.clone(),
                model: policy.model.clone(),
                command_argv,
                context_path,
                stdout_path: Some(stdout_path),
                stderr_path: Some(stderr_path),
                exit_status: output.status.code(),
                output_sha256: Some(stdout_sha),
                edits_count: 0,
                edit_paths: Vec::new(),
                failed_verifier_count: context.failed_verifiers.len(),
                diff_context_bytes: context.git_diff.len(),
                error_summary: Some(error),
                created_unix_ms: started_unix_ms,
            },
            edits: Vec::new(),
        }),
    }
}

pub fn parse_provider_repair_edits(
    response_text: &str,
    repo_path: &Path,
) -> Result<Vec<CodingMissionControlledEdit>, String> {
    let payload = parse_provider_repair_payload(response_text)?;
    let edits = match payload {
        ProviderRepairPayload::Single(entry) => vec![provider_payload_entry_to_edit(entry)?],
        ProviderRepairPayload::Multi { edits } => {
            if edits.is_empty() {
                return Err("provider repair edit set must not be empty".to_string());
            }
            edits
                .into_iter()
                .map(provider_payload_entry_to_edit)
                .collect::<Result<Vec<_>, _>>()?
        }
        ProviderRepairPayload::FileMap { files, reason_code } => {
            if files.is_empty() {
                return Err("provider repair file map must not be empty".to_string());
            }
            files
                .into_iter()
                .map(|(relative_path, contents)| {
                    provider_payload_entry_to_edit(ProviderRepairPayloadEntry {
                        relative_path,
                        contents,
                        reason_code: reason_code.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        ProviderRepairPayload::UnifiedDiff { diff, reason_code } => {
            provider_unified_diff_to_edits(repo_path, &diff, reason_code)?
        }
    };
    if edits.is_empty() {
        return Err("provider repair produced no edits".to_string());
    }
    Ok(edits)
}

fn build_provider_repair_context(
    state: &CodingMissionState,
    job_id: &str,
    attempt_index: u32,
    created_unix_ms: u64,
) -> Result<AutonomousCodingProviderRepairContext> {
    let failed_verifiers = state
        .command_evidence
        .iter()
        .rev()
        .filter(|evidence| {
            evidence.reason_code.starts_with("coding_verifier")
                && evidence.status == CodingWorkspaceCommandStatus::Failed
        })
        .take(4)
        .map(|evidence| AutonomousCodingVerifierFailureContext {
            argv: evidence.argv.clone(),
            exit_status: evidence.exit_status,
            stdout: read_artifact_snippet(evidence.stdout_path.as_path(), 2_000),
            stderr: read_artifact_snippet(evidence.stderr_path.as_path(), 4_000),
        })
        .collect::<Vec<_>>();
    let changed_files_raw = git_lines(&state.repo_path, &["status", "--porcelain"])?;
    let git_diff = git_snippet(&state.repo_path, &["diff", "--", "."], 16_000)?;
    Ok(AutonomousCodingProviderRepairContext {
        schema_version: 1,
        job_id: job_id.to_string(),
        mission_id: state.mission_id.clone(),
        attempt_index,
        goal: state.goal.clone(),
        repo_path: state.repo_path.clone(),
        issue_url: state.issue_url.clone(),
        verifier_commands: state.verifier_commands.clone(),
        failed_verifiers,
        changed_files: parse_git_status_changed_file_names(&changed_files_raw),
        git_diff,
        created_unix_ms,
    })
}

#[allow(clippy::too_many_arguments)]
fn provider_repair_failure_with_artifacts(
    policy: &AutonomousCodingProviderRepairPolicy,
    attempt_index: u32,
    context_path: PathBuf,
    command_argv: Vec<String>,
    stdout_path: Option<PathBuf>,
    stderr_path: Option<PathBuf>,
    exit_status: Option<i32>,
    failed_verifier_count: usize,
    diff_context_bytes: usize,
    reason_code: &str,
    error_summary: String,
    created_unix_ms: u64,
) -> AutonomousCodingProviderRepairRun {
    AutonomousCodingProviderRepairRun {
        evidence: AutonomousCodingProviderRepairEvidence {
            status: AutonomousCodingProviderRepairStatus::Failed,
            attempt_index,
            reason_code: reason_code.to_string(),
            provider: policy.provider.clone(),
            model: policy.model.clone(),
            command_argv,
            context_path,
            stdout_path,
            stderr_path,
            exit_status,
            output_sha256: None,
            edits_count: 0,
            edit_paths: Vec::new(),
            failed_verifier_count,
            diff_context_bytes,
            error_summary: Some(error_summary),
            created_unix_ms,
        },
        edits: Vec::new(),
    }
}

fn parse_provider_repair_payload(response_text: &str) -> Result<ProviderRepairPayload, String> {
    let trimmed = response_text.trim();
    match serde_json::from_str::<ProviderRepairPayload>(trimmed) {
        Ok(payload) => Ok(payload),
        Err(error) => {
            for candidate in provider_json_candidates(trimmed) {
                if candidate.trim() == trimmed {
                    continue;
                }
                if let Ok(payload) = serde_json::from_str::<ProviderRepairPayload>(candidate.trim())
                {
                    return Ok(payload);
                }
            }
            Err(format!(
                "provider repair response was not valid JSON: {error}"
            ))
        }
    }
}

fn provider_payload_entry_to_edit(
    payload: ProviderRepairPayloadEntry,
) -> Result<CodingMissionControlledEdit, String> {
    let relative_path = validate_relative_repo_path(payload.relative_path.trim())?;
    let mut contents = payload.contents;
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    let reason_code = payload
        .reason_code
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "provider_repair_edit".to_string());
    Ok(CodingMissionControlledEdit {
        relative_path,
        contents,
        reason_code,
    })
}

fn provider_unified_diff_to_edits(
    repo_path: &Path,
    diff: &str,
    reason_code: Option<String>,
) -> Result<Vec<CodingMissionControlledEdit>, String> {
    if diff.trim().is_empty() {
        return Err("provider repair diff must not be empty".to_string());
    }
    let patches = parse_provider_unified_diff(diff)?;
    let mut edits = Vec::with_capacity(patches.len());
    let reason_code = reason_code
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "provider_repair_unified_diff".to_string());
    for patch in patches {
        let relative_path = validate_relative_repo_path(&patch.path)?;
        let absolute = normalize_path_lexically(&repo_path.join(&relative_path));
        if !absolute.starts_with(repo_path) {
            return Err(format!(
                "provider repair diff path escapes repo: {}",
                relative_path.display()
            ));
        }
        if !absolute.is_file() {
            return Err(format!(
                "provider repair diff target is not an existing file: {}",
                relative_path.display()
            ));
        }
        let source = std::fs::read_to_string(&absolute)
            .map_err(|error| format!("failed to read {}: {error}", absolute.display()))?;
        let applied = apply_provider_unified_file_patch(&source, &patch)?;
        edits.push(CodingMissionControlledEdit {
            relative_path,
            contents: applied.updated,
            reason_code: reason_code.clone(),
        });
    }
    Ok(edits)
}

#[derive(Debug)]
struct ProviderUnifiedFilePatch {
    path: String,
    hunks: Vec<ProviderUnifiedHunk>,
}

#[derive(Debug)]
struct ProviderUnifiedHunk {
    old_start: usize,
    lines: Vec<ProviderUnifiedHunkLine>,
}

#[derive(Debug)]
enum ProviderUnifiedHunkLine {
    Context(String),
    Remove(String),
    Add(String),
}

#[derive(Debug)]
struct ProviderUnifiedApplyResult {
    updated: String,
}

fn parse_provider_unified_diff(diff: &str) -> Result<Vec<ProviderUnifiedFilePatch>, String> {
    let lines = diff.lines().collect::<Vec<_>>();
    let mut patches = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        if !is_unified_file_header(&lines, index) {
            index = index.saturating_add(1);
            continue;
        }

        let old_path = parse_unified_header_path(lines[index], "--- ")?;
        let new_path = parse_unified_header_path(lines[index + 1], "+++ ")?;
        if old_path == "/dev/null" || new_path == "/dev/null" {
            return Err(
                "provider repair unified diff only supports existing-file modifications"
                    .to_string(),
            );
        }
        let old_normalized = normalize_unified_diff_path(&old_path)?;
        let path = normalize_unified_diff_path(&new_path)?;
        if old_normalized != path {
            return Err(format!(
                "provider repair unified diff does not support renames: '{}' -> '{}'",
                old_normalized, path
            ));
        }
        index = index.saturating_add(2);

        let mut hunks = Vec::new();
        while index < lines.len() {
            if is_unified_file_header(&lines, index) {
                break;
            }
            let line = lines[index];
            if !line.starts_with("@@") {
                index = index.saturating_add(1);
                continue;
            }

            let old_start = parse_unified_hunk_old_start(line)?;
            index = index.saturating_add(1);
            let mut hunk_lines = Vec::new();
            while index < lines.len()
                && !lines[index].starts_with("@@")
                && !is_unified_file_boundary(&lines, index)
            {
                let hunk_line = lines[index];
                if hunk_line.starts_with("\\ ") {
                    index = index.saturating_add(1);
                    continue;
                }
                let Some(prefix) = hunk_line.chars().next() else {
                    return Err("malformed provider repair diff hunk line".to_string());
                };
                let text = hunk_line
                    .get(prefix.len_utf8()..)
                    .unwrap_or_default()
                    .to_string();
                match prefix {
                    ' ' => hunk_lines.push(ProviderUnifiedHunkLine::Context(text)),
                    '-' => hunk_lines.push(ProviderUnifiedHunkLine::Remove(text)),
                    '+' => hunk_lines.push(ProviderUnifiedHunkLine::Add(text)),
                    _ => {
                        return Err(format!(
                            "malformed provider repair diff hunk prefix '{}'",
                            prefix
                        ));
                    }
                }
                index = index.saturating_add(1);
            }
            if hunk_lines.is_empty() {
                return Err("provider repair diff hunk must include lines".to_string());
            }
            hunks.push(ProviderUnifiedHunk {
                old_start,
                lines: hunk_lines,
            });
        }

        if hunks.is_empty() {
            return Err(format!("provider repair diff file '{}' has no hunks", path));
        }
        patches.push(ProviderUnifiedFilePatch { path, hunks });
    }

    if patches.is_empty() {
        return Err("provider repair diff must include a file patch".to_string());
    }
    Ok(patches)
}

fn apply_provider_unified_file_patch(
    source: &str,
    patch: &ProviderUnifiedFilePatch,
) -> Result<ProviderUnifiedApplyResult, String> {
    let mut lines = source.split('\n').map(str::to_string).collect::<Vec<_>>();
    let mut offset: isize = 0;

    for (hunk_index, hunk) in patch.hunks.iter().enumerate() {
        let base_index = if hunk.old_start == 0 {
            0isize
        } else {
            hunk.old_start as isize - 1
        };
        let target_index = base_index.saturating_add(offset);
        if target_index < 0 {
            return Err(format!(
                "provider repair diff hunk {} resolves before start of file",
                hunk_index
            ));
        }
        let start = target_index as usize;
        if start > lines.len() {
            return Err(format!(
                "provider repair diff hunk {} starts past end of file",
                hunk_index
            ));
        }

        let mut expected_old = Vec::new();
        let mut replacement = Vec::new();
        for line in &hunk.lines {
            match line {
                ProviderUnifiedHunkLine::Context(text) => {
                    expected_old.push(text.clone());
                    replacement.push(text.clone());
                }
                ProviderUnifiedHunkLine::Remove(text) => {
                    expected_old.push(text.clone());
                }
                ProviderUnifiedHunkLine::Add(text) => {
                    replacement.push(text.clone());
                }
            }
        }

        let end = start.saturating_add(expected_old.len());
        if end > lines.len() {
            return Err(format!(
                "provider repair diff hunk {} extends past end of file",
                hunk_index
            ));
        }
        if lines[start..end] != expected_old {
            return Err(format!(
                "provider repair diff hunk {} did not match current file contents",
                hunk_index
            ));
        }

        let expected_len = expected_old.len();
        let replacement_len = replacement.len();
        lines.splice(start..end, replacement);
        offset = offset.saturating_add(replacement_len as isize - expected_len as isize);
    }

    Ok(ProviderUnifiedApplyResult {
        updated: lines.join("\n"),
    })
}

fn validate_relative_repo_path(path: &str) -> Result<PathBuf, String> {
    let relative_path = PathBuf::from(path.trim());
    if relative_path.as_os_str().is_empty() {
        return Err("provider repair path must not be empty".to_string());
    }
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("provider repair path escapes repo".to_string());
    }
    Ok(relative_path)
}

fn is_unified_file_header(lines: &[&str], index: usize) -> bool {
    lines
        .get(index)
        .is_some_and(|line| line.starts_with("--- "))
        && lines
            .get(index.saturating_add(1))
            .is_some_and(|line| line.starts_with("+++ "))
}

fn is_unified_file_boundary(lines: &[&str], index: usize) -> bool {
    lines
        .get(index)
        .is_some_and(|line| line.starts_with("diff --git "))
        || is_unified_file_header(lines, index)
}

fn parse_unified_header_path(line: &str, prefix: &str) -> Result<String, String> {
    let path = line
        .strip_prefix(prefix)
        .ok_or_else(|| format!("missing unified diff header prefix '{}'", prefix.trim()))?
        .split('\t')
        .next()
        .unwrap_or_default()
        .trim();
    if path.is_empty() {
        return Err("unified diff header path must not be empty".to_string());
    }
    Ok(path.to_string())
}

fn normalize_unified_diff_path(path: &str) -> Result<String, String> {
    let normalized = path
        .strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path)
        .trim();
    if normalized.is_empty() {
        return Err("unified diff path must not be empty".to_string());
    }
    Ok(normalized.to_string())
}

fn parse_unified_hunk_old_start(header: &str) -> Result<usize, String> {
    let body = header
        .strip_prefix("@@")
        .and_then(|rest| rest.split_once("@@").map(|(body, _)| body.trim()))
        .ok_or_else(|| format!("malformed unified diff hunk header '{}'", header))?;
    let mut ranges = body.split_whitespace();
    let old_range = ranges
        .next()
        .ok_or_else(|| format!("malformed unified diff hunk header '{}'", header))?;
    let new_range = ranges
        .next()
        .ok_or_else(|| format!("malformed unified diff hunk header '{}'", header))?;
    if !old_range.starts_with('-') || !new_range.starts_with('+') {
        return Err(format!("malformed unified diff hunk header '{}'", header));
    }
    let old_start = parse_unified_range_start(&old_range[1..], "old")?;
    parse_unified_range_start(&new_range[1..], "new")?;
    Ok(old_start)
}

fn parse_unified_range_start(range: &str, label: &str) -> Result<usize, String> {
    let (start, count) = range
        .split_once(',')
        .map_or((range, None), |(start, count)| (start, Some(count)));
    if start.is_empty() {
        return Err(format!("malformed unified diff {label} start"));
    }
    if let Some(count) = count {
        count
            .parse::<usize>()
            .map_err(|_| format!("malformed unified diff {label} count '{}'", count))?;
    }
    start
        .parse::<usize>()
        .map_err(|_| format!("malformed unified diff {label} start '{}'", start))
}

fn provider_repair_command_argv(command: &Path, args: &[String]) -> Vec<String> {
    let mut argv = Vec::with_capacity(args.len().saturating_add(1));
    argv.push(command.display().to_string());
    argv.extend(args.iter().cloned());
    argv
}

fn provider_json_candidates(response_text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.extend(provider_fenced_json_candidates(response_text));
    if let Some(candidate) = first_balanced_json_object(response_text) {
        candidates.push(candidate);
    }
    candidates
}

fn provider_fenced_json_candidates(response_text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut offset = 0;
    while let Some(start_rel) = response_text[offset..].find("```") {
        let content_start = offset + start_rel + 3;
        let Some(end_rel) = response_text[content_start..].find("```") else {
            break;
        };
        let content_end = content_start + end_rel;
        let raw = &response_text[content_start..content_end];
        let candidate = raw
            .strip_prefix("json")
            .or_else(|| raw.strip_prefix("JSON"))
            .unwrap_or(raw)
            .trim_start_matches(['\r', '\n', ' ']);
        if !candidate.trim().is_empty() {
            candidates.push(candidate.trim().to_string());
        }
        offset = content_end + 3;
    }
    candidates
}

fn first_balanced_json_object(response_text: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in response_text.char_indices() {
        if start.is_none() {
            if ch == '{' {
                start = Some(index);
                depth = 1;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let start = start.expect("start set");
                    let end = index + ch.len_utf8();
                    return Some(response_text[start..end].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn git_lines(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .with_context(|| format!("run git {} in {}", args.join(" "), repo_path.display()))?;
    if !output.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn git_snippet(repo_path: &Path, args: &[&str], max_chars: usize) -> Result<String> {
    let raw = git_lines(repo_path, args)?;
    Ok(truncate_chars(
        redact_secret_like_tokens(&raw).as_str(),
        max_chars,
    ))
}

fn parse_git_status_changed_file_names(raw_status: &str) -> Vec<String> {
    raw_status
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.rsplit_once(" -> ")
                .map(|(_from, to)| to)
                .unwrap_or(line)
                .to_string()
        })
        .collect()
}

fn read_artifact_snippet(path: &Path, max_chars: usize) -> String {
    let raw = std::fs::read_to_string(path).unwrap_or_default();
    truncate_chars(redact_secret_like_tokens(&raw).as_str(), max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated = value.chars().take(max_chars).collect::<String>();
    truncated.push_str("\n[truncated]");
    truncated
}

fn redact_secret_like_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| {
            if token.contains("sk-")
                || token.contains("OPENAI_API_KEY")
                || token.contains("ANTHROPIC_API_KEY")
                || token.contains("OPENROUTER_API_KEY")
                || token.contains("TAU_OPENROUTER_API_KEY")
            {
                "[REDACTED_API_KEY]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp_path, json.as_bytes())
        .with_context(|| format!("failed to write {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        )
    })
}
