//! Safety-signal and memory-recall helpers used by the runtime loop.

use serde_json::{json, Value};
use tau_ai::{Message, MessageRole};
use tau_memory::runtime::{cosine_similarity, embed_text_vector};

use crate::{
    collapse_whitespace, truncate_chars, AgentConfig, MemoryRecallMatch, FAILURE_SIGNAL_PHRASES,
};

pub(crate) fn assistant_text_suggests_failure(text: &str) -> bool {
    let normalized = collapse_whitespace(&text.to_lowercase());
    if normalized.trim().is_empty() {
        return true;
    }
    FAILURE_SIGNAL_PHRASES
        .iter()
        .any(|phrase| normalized.contains(phrase))
}

pub(crate) fn assistant_text_suggests_unverified_implementation_progress(
    user_prompt: &str,
    assistant_text: &str,
) -> bool {
    let normalized_prompt = collapse_whitespace(&user_prompt.to_lowercase());
    let normalized_assistant = collapse_whitespace(&assistant_text.to_lowercase());
    if normalized_prompt.trim().is_empty() || normalized_assistant.trim().is_empty() {
        return false;
    }
    if assistant_text_suggests_failure(&normalized_assistant) {
        return false;
    }
    if !user_prompt_requests_workspace_implementation(&normalized_prompt) {
        return false;
    }
    assistant_claims_implementation_progress(&normalized_assistant)
}

fn user_prompt_requests_workspace_implementation(normalized_prompt: &str) -> bool {
    let has_action = [
        "create",
        "build",
        "implement",
        "scaffold",
        "make",
        "develop",
        "fix",
        "add",
        "edit",
        "update",
        "refactor",
    ]
    .iter()
    .any(|term| normalized_prompt.contains(term));
    if !has_action {
        return false;
    }
    [
        "game",
        "app",
        "application",
        "site",
        "website",
        "page",
        "component",
        "feature",
        "ui",
        "scene",
        "phaser",
        "phaserjs",
        "project",
        "prototype",
        "script",
    ]
    .iter()
    .any(|term| normalized_prompt.contains(term))
}

fn assistant_claims_implementation_progress(normalized_assistant: &str) -> bool {
    [
        "going well",
        "core systems are in place",
        "systems are in place",
        "implemented",
        "built",
        "created",
        "scaffolded",
        "wired up",
        "hooked up",
        "finishing",
        "wrapping up",
        "completed",
    ]
    .iter()
    .any(|term| normalized_assistant.contains(term))
}

#[derive(Debug, Clone)]
struct MemoryEmbeddingApiConfig {
    model: String,
    api_base: String,
    api_key: String,
}

pub(crate) async fn retrieve_memory_matches(
    history: &[Message],
    query: &str,
    limit: usize,
    dimensions: usize,
    min_similarity: f32,
    config: &AgentConfig,
) -> Vec<MemoryRecallMatch> {
    if limit == 0 {
        return Vec::new();
    }
    let candidates = history
        .iter()
        .filter_map(|message| match message.role {
            MessageRole::User | MessageRole::Assistant => {
                let text = message.text_content();
                if text.trim().is_empty() {
                    None
                } else {
                    Some((message.role, text))
                }
            }
            MessageRole::Tool | MessageRole::System => None,
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Vec::new();
    }

    let api_embeddings = if let Some(api_config) = resolve_memory_embedding_api_config(config) {
        let mut inputs = Vec::with_capacity(candidates.len().saturating_add(1));
        inputs.push(query.to_string());
        inputs.extend(candidates.iter().map(|(_, text)| text.clone()));
        match embed_text_vectors_via_api(&inputs, dimensions, &api_config).await {
            Ok(vectors) if vectors.len() == inputs.len() => Some(vectors),
            _ => None,
        }
    } else {
        None
    };

    let (query_embedding, candidate_embeddings) = if let Some(vectors) = api_embeddings {
        let query_embedding = vectors.first().cloned().unwrap_or_default();
        let candidate_embeddings = vectors.into_iter().skip(1).collect::<Vec<_>>();
        (query_embedding, candidate_embeddings)
    } else {
        let query_embedding = embed_text_vector(query, dimensions);
        let candidate_embeddings = candidates
            .iter()
            .map(|(_, text)| embed_text_vector(text, dimensions))
            .collect::<Vec<_>>();
        (query_embedding, candidate_embeddings)
    };
    if query_embedding.iter().all(|component| *component == 0.0) {
        return Vec::new();
    }

    let mut matches = candidates
        .into_iter()
        .zip(candidate_embeddings.into_iter())
        .filter_map(|((role, text), candidate_embedding)| {
            let score = cosine_similarity(&query_embedding, &candidate_embedding);
            if score >= min_similarity {
                Some(MemoryRecallMatch { score, role, text })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| right.score.total_cmp(&left.score));
    matches.truncate(limit);
    matches
}

fn resolve_memory_embedding_api_config(config: &AgentConfig) -> Option<MemoryEmbeddingApiConfig> {
    let model = config.memory_embedding_model.as_deref()?.trim();
    let api_key = config.memory_embedding_api_key.as_deref()?.trim();
    if model.is_empty() || api_key.is_empty() {
        return None;
    }
    let api_base = config
        .memory_embedding_api_base
        .as_deref()
        .unwrap_or("https://api.openai.com/v1")
        .trim()
        .trim_end_matches('/')
        .to_string();
    if api_base.is_empty() {
        return None;
    }
    Some(MemoryEmbeddingApiConfig {
        model: model.to_string(),
        api_base,
        api_key: api_key.to_string(),
    })
}

async fn embed_text_vectors_via_api(
    inputs: &[String],
    dimensions: usize,
    config: &MemoryEmbeddingApiConfig,
) -> Result<Vec<Vec<f32>>, String> {
    if inputs.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/embeddings", config.api_base))
        .bearer_auth(config.api_key.as_str())
        .json(&json!({
            "model": config.model,
            "input": inputs,
        }))
        .send()
        .await
        .map_err(|error| format!("embedding request failed: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "embedding request failed with status {}: {}",
            status.as_u16(),
            truncate_chars(&body, 240)
        ));
    }

    let payload = response
        .json::<Value>()
        .await
        .map_err(|error| format!("failed to parse embedding response json: {error}"))?;
    let data = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "embedding response missing data array".to_string())?;
    if data.len() != inputs.len() {
        return Err(format!(
            "embedding response size mismatch: expected {}, got {}",
            inputs.len(),
            data.len()
        ));
    }

    let mut vectors = Vec::with_capacity(data.len());
    for item in data {
        let raw_embedding = item
            .get("embedding")
            .and_then(Value::as_array)
            .ok_or_else(|| "embedding item missing embedding array".to_string())?;
        let parsed = raw_embedding
            .iter()
            .map(|component| {
                component
                    .as_f64()
                    .map(|value| value as f32)
                    .ok_or_else(|| "embedding component must be numeric".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        vectors.push(resize_and_normalize_embedding(&parsed, dimensions));
    }

    Ok(vectors)
}

fn resize_and_normalize_embedding(values: &[f32], dimensions: usize) -> Vec<f32> {
    let dimensions = dimensions.max(1);
    let mut resized = vec![0.0f32; dimensions];
    for (index, value) in values.iter().enumerate() {
        let bucket = index % dimensions;
        resized[bucket] += *value;
    }

    let magnitude = resized
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if magnitude > 0.0 {
        for component in &mut resized {
            *component /= magnitude;
        }
    }
    resized
}
