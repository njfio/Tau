use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tau_ai::{
    ChatRequest, ChatUsage, LlmClient, Message, OpenAiAuthScheme, OpenAiClient, OpenAiConfig,
    PromptCacheConfig, Provider, ToolChoice,
};
use tau_provider::{
    missing_provider_api_key_message, provider_api_key_candidates_with_inputs,
    resolve_non_empty_secret_with_source,
};
use tau_runtime::{parse_provider_repair_edits, AutonomousCodingProviderRepairContext};

const DEFAULT_OPENROUTER_API_BASE: &str = "https://openrouter.ai/api/v1";
const DEFAULT_OPENROUTER_MODEL: &str = "openrouter/openai/gpt-4.1-mini";
const PROVIDER_NAME: &str = "openrouter";

const MODEL_ENV_NAMES: &[&str] = &[
    "TAU_PROVIDER_REPAIR_MODEL",
    "TAU_AUTONOMOUS_CODING_REPAIR_MODEL",
    "TAU_PROVIDER_PROOF_MODEL",
    "TAU_OPENROUTER_MODEL",
];

const API_BASE_ENV_NAMES: &[&str] = &[
    "TAU_PROVIDER_REPAIR_API_BASE",
    "TAU_AUTONOMOUS_CODING_REPAIR_API_BASE",
    "TAU_OPENROUTER_API_BASE",
];

const API_KEY_ENV_NAMES: &[&str] = &[
    "OPENROUTER_API_KEY",
    "TAU_OPENROUTER_API_KEY",
    "OPENAI_API_KEY",
    "TAU_API_KEY",
];

#[derive(Debug, Clone)]
pub struct OpenrouterRepairAdapterConfig {
    pub context_path: Option<PathBuf>,
    pub env_file: Option<PathBuf>,
    pub model: Option<String>,
    pub api_base: Option<String>,
    pub request_timeout_ms: u64,
    pub max_tokens: u32,
    pub max_retries: usize,
}

#[derive(Debug, Clone)]
struct ResolvedOpenrouterSecret {
    value: String,
    source: String,
}

#[derive(Debug, Clone)]
struct ValidatedProviderRepairJson {
    json: String,
    edit_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OpenrouterRepairAdapterMetadata {
    schema_version: u32,
    status: String,
    provider: String,
    model: String,
    api_model: String,
    api_base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_source: Option<String>,
    context_path: PathBuf,
    job_id: String,
    mission_id: String,
    attempt_index: u32,
    response_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<ChatUsage>,
    edit_count: usize,
    edit_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_summary: Option<String>,
    created_unix_ms: u64,
}

pub async fn run_openrouter_repair_adapter(config: OpenrouterRepairAdapterConfig) -> Result<()> {
    let env_file = resolve_openrouter_env_file(config.env_file);
    let context_path = resolve_context_path(config.context_path)?;
    let context = read_context(&context_path)?;
    let metadata_path = metadata_path_for_context(&context_path);
    let model = resolve_openrouter_repair_model(config.model.as_deref(), env_file.as_deref());
    let api_base = resolve_openrouter_api_base(config.api_base.as_deref(), env_file.as_deref());
    let api_model = openrouter_api_model_id(&model)?;
    let secret = resolve_openrouter_api_key(env_file.as_deref())?;
    let created_unix_ms = now_unix_ms();

    let response = call_openrouter_repair_model(
        &context,
        &api_base,
        &api_model,
        &secret.value,
        config.request_timeout_ms,
        config.max_tokens,
        config.max_retries,
    )
    .await;

    match response {
        Ok(response) => {
            let response_text = response.message.text_content();
            let response_sha = sha256_hex(response_text.as_bytes());
            let validated = strict_provider_repair_json(&response_text, &context.repo_path)
                .inspect_err(|error| {
                    let metadata = OpenrouterRepairAdapterMetadata {
                        schema_version: 1,
                        status: "rejected".to_string(),
                        provider: PROVIDER_NAME.to_string(),
                        model: model.clone(),
                        api_model: api_model.clone(),
                        api_base: api_base.clone(),
                        auth_source: Some(secret.source.clone()),
                        context_path: context_path.clone(),
                        job_id: context.job_id.clone(),
                        mission_id: context.mission_id.clone(),
                        attempt_index: context.attempt_index,
                        response_bytes: response_text.len(),
                        response_sha256: Some(response_sha.clone()),
                        finish_reason: response.finish_reason.clone(),
                        usage: Some(response.usage.clone()),
                        edit_count: 0,
                        edit_paths: Vec::new(),
                        error_summary: Some(error.to_string()),
                        created_unix_ms,
                    };
                    let _ = write_json_atomic(&metadata_path, &metadata);
                })?;

            let metadata = OpenrouterRepairAdapterMetadata {
                schema_version: 1,
                status: "validated".to_string(),
                provider: PROVIDER_NAME.to_string(),
                model,
                api_model,
                api_base,
                auth_source: Some(secret.source),
                context_path,
                job_id: context.job_id,
                mission_id: context.mission_id,
                attempt_index: context.attempt_index,
                response_bytes: response_text.len(),
                response_sha256: Some(response_sha),
                finish_reason: response.finish_reason,
                usage: Some(response.usage),
                edit_count: validated.edit_paths.len(),
                edit_paths: validated.edit_paths,
                error_summary: None,
                created_unix_ms,
            };
            write_json_atomic(&metadata_path, &metadata)?;
            println!("{}", validated.json);
            Ok(())
        }
        Err(error) => {
            let metadata = OpenrouterRepairAdapterMetadata {
                schema_version: 1,
                status: "failed".to_string(),
                provider: PROVIDER_NAME.to_string(),
                model,
                api_model,
                api_base,
                auth_source: Some(secret.source),
                context_path,
                job_id: context.job_id,
                mission_id: context.mission_id,
                attempt_index: context.attempt_index,
                response_bytes: 0,
                response_sha256: None,
                finish_reason: None,
                usage: None,
                edit_count: 0,
                edit_paths: Vec::new(),
                error_summary: Some(error.to_string()),
                created_unix_ms,
            };
            let _ = write_json_atomic(&metadata_path, &metadata);
            Err(error)
        }
    }
}

pub fn resolve_openrouter_env_file(configured: Option<PathBuf>) -> Option<PathBuf> {
    configured.or_else(nearest_dotenv_path_from_current_dir)
}

pub fn resolve_openrouter_repair_model(
    configured: Option<&str>,
    env_file: Option<&Path>,
) -> String {
    configured
        .and_then(trimmed_non_empty)
        .map(str::to_string)
        .or_else(|| first_env_value(MODEL_ENV_NAMES, env_file))
        .unwrap_or_else(|| DEFAULT_OPENROUTER_MODEL.to_string())
}

fn resolve_openrouter_api_base(configured: Option<&str>, env_file: Option<&Path>) -> String {
    configured
        .and_then(trimmed_non_empty)
        .map(str::to_string)
        .or_else(|| first_env_value(API_BASE_ENV_NAMES, env_file))
        .unwrap_or_else(|| DEFAULT_OPENROUTER_API_BASE.to_string())
}

pub fn openrouter_api_model_id(model_ref: &str) -> Result<String> {
    let model = model_ref.trim();
    if model.is_empty() {
        bail!("OpenRouter model must not be empty");
    }
    Ok(model
        .strip_prefix("openrouter/")
        .unwrap_or(model)
        .to_string())
}

fn resolve_context_path(configured: Option<PathBuf>) -> Result<PathBuf> {
    configured
        .or_else(|| std::env::var_os("TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT").map(PathBuf::from))
        .ok_or_else(|| {
            anyhow!(
                "missing repair context path. Pass --context or set TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT"
            )
        })
}

fn read_context(path: &Path) -> Result<AutonomousCodingProviderRepairContext> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read repair context {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse repair context {}", path.display()))
}

fn resolve_openrouter_api_key(env_file: Option<&Path>) -> Result<ResolvedOpenrouterSecret> {
    for name in API_KEY_ENV_NAMES {
        if let Some(value) = non_empty_env_var(name) {
            return Ok(ResolvedOpenrouterSecret {
                value,
                source: (*name).to_string(),
            });
        }
        if let Some(path) = env_file {
            if let Some(value) = dotenv_var_from_path(path, name) {
                return Ok(ResolvedOpenrouterSecret {
                    value,
                    source: format!("{}:{}", path.display(), name),
                });
            }
        }
    }

    let candidates =
        provider_api_key_candidates_with_inputs(Provider::OpenRouter, None, None, None, None);
    if let Some((value, source)) = resolve_non_empty_secret_with_source(candidates) {
        return Ok(ResolvedOpenrouterSecret { value, source });
    }

    bail!("{}", missing_provider_api_key_message(Provider::OpenRouter))
}

#[allow(clippy::too_many_arguments)]
async fn call_openrouter_repair_model(
    context: &AutonomousCodingProviderRepairContext,
    api_base: &str,
    api_model: &str,
    api_key: &str,
    request_timeout_ms: u64,
    max_tokens: u32,
    max_retries: usize,
) -> Result<tau_ai::ChatResponse> {
    let client = OpenAiClient::new(OpenAiConfig {
        api_base: api_base.to_string(),
        api_key: api_key.to_string(),
        organization: None,
        request_timeout_ms: request_timeout_ms.max(1),
        max_retries,
        retry_budget_ms: 0,
        retry_jitter: false,
        auth_scheme: OpenAiAuthScheme::Bearer,
        api_version: None,
    })?;
    client
        .complete(ChatRequest {
            model: api_model.to_string(),
            messages: vec![
                Message::system(provider_repair_system_prompt()),
                Message::user(provider_repair_user_prompt(context)?),
            ],
            tools: Vec::new(),
            tool_choice: Some(ToolChoice::None),
            json_mode: true,
            max_tokens: Some(max_tokens.max(1)),
            temperature: Some(0.0),
            prompt_cache: PromptCacheConfig::default(),
        })
        .await
        .map_err(Into::into)
}

fn provider_repair_system_prompt() -> String {
    [
        "You are Tau's provider repair adapter for an autonomous coding job.",
        "Return only one JSON object and no Markdown.",
        "Use one of these schemas:",
        r#"{"relative_path":"path/in/repo","contents":"full file contents","reason_code":"short_reason"}"#,
        r#"{"edits":[{"relative_path":"path/in/repo","contents":"full file contents","reason_code":"short_reason"}]}"#,
        r#"{"files":{"path/in/repo":"full file contents"},"reason_code":"short_reason"}"#,
        r#"{"diff":"--- a/path\n+++ b/path\n@@ ...","reason_code":"short_reason"}"#,
        "Paths must be relative to the repository, must not include '..', and must not be absolute.",
        "Make the smallest targeted repair needed for the failed verifier output.",
    ]
    .join("\n")
}

fn provider_repair_user_prompt(context: &AutonomousCodingProviderRepairContext) -> Result<String> {
    Ok(format!(
        "Repair this Tau autonomous coding job.\n\nContext JSON:\n{}",
        serde_json::to_string_pretty(context)?
    ))
}

fn strict_provider_repair_json(
    response_text: &str,
    repo_path: &Path,
) -> Result<ValidatedProviderRepairJson> {
    let mut errors = Vec::new();
    for candidate in provider_json_candidates(response_text) {
        match parse_provider_repair_edits(&candidate, repo_path) {
            Ok(edits) => {
                let edit_paths = edits
                    .into_iter()
                    .map(|edit| edit.relative_path.display().to_string())
                    .collect::<Vec<_>>();
                return Ok(ValidatedProviderRepairJson {
                    json: candidate,
                    edit_paths,
                });
            }
            Err(error) => errors.push(error),
        }
    }
    let detail = errors
        .into_iter()
        .next()
        .unwrap_or_else(|| "no JSON object found in provider response".to_string());
    bail!("OpenRouter repair response did not match Tau edit/diff contract: {detail}")
}

fn provider_json_candidates(response_text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    push_candidate(response_text.trim(), &mut seen, &mut candidates);
    for block in fenced_json_blocks(response_text) {
        push_candidate(block.trim(), &mut seen, &mut candidates);
    }
    if let Some(object) = first_balanced_json_object(response_text) {
        push_candidate(object.trim(), &mut seen, &mut candidates);
    }
    candidates
}

fn push_candidate(candidate: &str, seen: &mut BTreeSet<String>, candidates: &mut Vec<String>) {
    if candidate.is_empty() {
        return;
    }
    let value = candidate.to_string();
    if seen.insert(value.clone()) {
        candidates.push(value);
    }
}

fn fenced_json_blocks(response_text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut active = false;
    let mut current = Vec::new();
    for line in response_text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if active {
                blocks.push(current.join("\n"));
                current.clear();
                active = false;
            } else {
                active = true;
            }
            continue;
        }
        if active {
            current.push(line);
        }
    }
    blocks
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

        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        if ch == '{' {
            depth = depth.saturating_add(1);
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let start = start?;
                let end = index + ch.len_utf8();
                return Some(response_text[start..end].to_string());
            }
        }
    }
    None
}

fn first_env_value(names: &[&str], env_file: Option<&Path>) -> Option<String> {
    names
        .iter()
        .find_map(|name| env_or_dotenv_var(name, env_file))
}

fn env_or_dotenv_var(name: &str, env_file: Option<&Path>) -> Option<String> {
    non_empty_env_var(name)
        .or_else(|| env_file.and_then(|path| dotenv_var_from_path(path, name)))
        .or_else(|| nearest_dotenv_var(name))
}

fn non_empty_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn nearest_dotenv_path_from_current_dir() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    for dir in current_dir.ancestors() {
        let dotenv_path = dir.join(".env");
        if dotenv_path.is_file() {
            return Some(dotenv_path);
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    None
}

fn nearest_dotenv_var(name: &str) -> Option<String> {
    nearest_dotenv_path_from_current_dir().and_then(|path| dotenv_var_from_path(&path, name))
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

fn metadata_path_for_context(context_path: &Path) -> PathBuf {
    let file_name = context_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("provider-repair-context.json");
    context_path.with_file_name(format!("{file_name}.openrouter-call.json"))
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn trimmed_non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_3803_resolves_model_from_explicit_env_file() {
        let temp = tempfile::tempdir().expect("tempdir");
        let env_file = temp.path().join(".env");
        fs::write(
            &env_file,
            "TAU_PROVIDER_PROOF_MODEL=openrouter/qwen/qwen-3.7-max\n",
        )
        .expect("env write");

        assert_eq!(
            resolve_openrouter_repair_model(None, Some(&env_file)),
            "openrouter/qwen/qwen-3.7-max"
        );
    }

    #[test]
    fn spec_3803_strips_openrouter_prefix_for_api_model_id() {
        assert_eq!(
            openrouter_api_model_id("openrouter/deepseek/deepseek-chat-v3-0324").expect("model"),
            "deepseek/deepseek-chat-v3-0324"
        );
        assert_eq!(
            openrouter_api_model_id("qwen/qwen-3.7-max").expect("model"),
            "qwen/qwen-3.7-max"
        );
    }

    #[test]
    fn spec_3803_extracts_fenced_json_and_validates_contract() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo_path = temp.path();
        let response = r#"Here is the repair:
```json
{"edits":[{"relative_path":"src/lib.rs","contents":"pub fn value() -> u8 { 1 }\n","reason_code":"fix_value"}]}
```
"#;

        let validated = strict_provider_repair_json(response, repo_path).expect("valid contract");
        assert_eq!(validated.edit_paths, vec!["src/lib.rs"]);
        assert!(validated.json.contains("\"edits\""));
    }
}
