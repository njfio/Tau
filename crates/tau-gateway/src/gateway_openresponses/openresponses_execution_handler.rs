//! OpenResponses execution handler.

use super::*;
use std::collections::HashSet;

pub(super) async fn execute_openresponses_request(
    state: Arc<GatewayOpenResponsesServerState>,
    request: OpenResponsesRequest,
    stream_sender: Option<mpsc::UnboundedSender<SseFrame>>,
) -> Result<OpenResponsesExecutionResult, OpenResponsesApiError> {
    let mut translated = translate_openresponses_request(&request, state.config.max_input_chars)?;
    if request.model.is_some() {
        translated.ignored_fields.push("model".to_string());
    }

    let response_id = state.next_response_id();
    let created = current_unix_timestamp();

    if let Some(sender) = &stream_sender {
        let _ = sender.send(SseFrame::Json {
            event: "response.created",
            payload: json!({
                "type": "response.created",
                "response": {
                    "id": response_id,
                    "object": "response",
                    "status": "in_progress",
                    "model": state.config.model,
                    "created": created,
                }
            }),
        });
    }

    let resolved_system_prompt = state.resolved_system_prompt();
    let mut agent = Agent::new(
        state.config.client.clone(),
        AgentConfig {
            model: state.config.model.clone(),
            model_input_cost_per_million: state.config.model_input_cost_per_million,
            model_cached_input_cost_per_million: state.config.model_cached_input_cost_per_million,
            model_output_cost_per_million: state.config.model_output_cost_per_million,
            system_prompt: resolved_system_prompt.clone(),
            // Codex CLI backend: one turn, no agent-level timeout or retries.
            // The codex CLI runs its own agent loop with --full-auto.
            max_turns: 1,
            request_timeout_ms: None, // No timeout — codex manages its own
            request_max_retries: 0,   // No retries — one codex invocation per request
            max_tokens: request.max_tokens,
            // `translate_openresponses_request` already enforces `max_input_chars` for the
            // inbound payload. Reusing that transport guardrail as an agent token budget
            // incorrectly counts system prompt and persisted session history.
            max_estimated_input_tokens: None,
            max_estimated_total_tokens: None,
            ..AgentConfig::default()
        },
    );
    state.config.tool_registrar.register(&mut agent);

    let usage = Arc::new(Mutex::new(OpenResponsesUsageSummary::default()));
    let finish_reason = Arc::new(Mutex::new(None::<String>));
    let event_stream_sender = stream_sender.clone();
    agent.subscribe({
        let usage = usage.clone();
        let finish_reason = finish_reason.clone();
        move |event| {
            match event {
                AgentEvent::TurnEnd {
                    usage: turn_usage,
                    finish_reason: turn_finish_reason,
                    ..
                } => {
                    if let Ok(mut guard) = usage.lock() {
                        guard.input_tokens =
                            guard.input_tokens.saturating_add(turn_usage.input_tokens);
                        guard.output_tokens =
                            guard.output_tokens.saturating_add(turn_usage.output_tokens);
                        guard.total_tokens =
                            guard.total_tokens.saturating_add(turn_usage.total_tokens);
                    }
                    if let Ok(mut guard) = finish_reason.lock() {
                        *guard = turn_finish_reason.clone();
                    }
                    // Stream usage update to TUI
                    if let Some(sender) = &event_stream_sender {
                        let _ = sender.send(SseFrame::Json {
                            event: "response.usage.delta",
                            payload: json!({
                                "type": "response.usage.delta",
                                "usage": {
                                    "input_tokens": turn_usage.input_tokens,
                                    "output_tokens": turn_usage.output_tokens,
                                    "total_tokens": turn_usage.total_tokens,
                                }
                            }),
                        });
                    }
                }
                AgentEvent::CostUpdated {
                    cumulative_cost_usd,
                    ..
                } => {
                    if let Some(sender) = &event_stream_sender {
                        let _ = sender.send(SseFrame::Json {
                            event: "response.cost.delta",
                            payload: json!({
                                "type": "response.cost.delta",
                                "cumulative_cost_usd": cumulative_cost_usd,
                            }),
                        });
                    }
                }
                AgentEvent::ToolExecutionStart {
                    tool_name,
                    arguments,
                    ..
                } => {
                    if let Some(sender) = &event_stream_sender {
                        let _ = sender.send(SseFrame::Json {
                            event: "response.tool.start",
                            payload: json!({
                                "type": "response.tool.start",
                                "tool_name": tool_name,
                                "arguments_preview": arguments.to_string().chars().take(200).collect::<String>(),
                            }),
                        });
                    }
                }
                AgentEvent::ToolExecutionEnd {
                    tool_name,
                    result,
                    ..
                } => {
                    if let Some(sender) = &event_stream_sender {
                        let output_preview = result.content.as_str()
                            .unwrap_or("")
                            .chars()
                            .take(200)
                            .collect::<String>();
                        let _ = sender.send(SseFrame::Json {
                            event: "response.tool.end",
                            payload: json!({
                                "type": "response.tool.end",
                                "tool_name": tool_name,
                                "success": !result.is_error,
                                "output_preview": output_preview,
                            }),
                        });
                    }
                }
                AgentEvent::MessageAdded { message } => {
                    // Only stream assistant messages to the TUI — hide internal
                    // user/system messages (Ralph loop nudges, replans, etc.)
                    if message.role == tau_ai::MessageRole::Assistant {
                        if let Some(sender) = &event_stream_sender {
                            let text = message.text_content();
                            if !text.is_empty() {
                                let _ = sender.send(SseFrame::Json {
                                    event: "response.message.added",
                                    payload: json!({
                                        "type": "response.message.added",
                                        "role": "assistant",
                                        "text": text,
                                    }),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    let session_path = gateway_session_path(&state.config.state_dir, &translated.session_key);
    let mut session_runtime = Some(
        initialize_gateway_session_runtime(
            &session_path,
            &resolved_system_prompt,
            state.config.session_lock_wait_ms,
            state.config.session_lock_stale_ms,
            &mut agent,
        )
        .map_err(|error| {
            OpenResponsesApiError::internal(format!(
                "failed to initialize gateway session runtime: {error}"
            ))
        })?,
    );

    let auto_selected = select_gateway_auto_skills(
        &state.config.available_skills,
        &state.config.explicit_skill_names,
        &translated.prompt,
    );
    let effective_system_prompt = if auto_selected.is_empty() {
        resolved_system_prompt.clone()
    } else {
        augment_gateway_system_prompt(&resolved_system_prompt, &auto_selected)
    };
    let _ = agent.replace_system_prompt(effective_system_prompt);

    let start_index = agent.messages().len();

    let pre_prompt_cost = agent.cost_snapshot();
    let stream_handler = stream_sender.as_ref().map(|sender| {
        let sender = sender.clone();
        let response_id = response_id.clone();
        Arc::new(move |delta: String| {
            if delta.is_empty() {
                return;
            }
            let _ = sender.send(SseFrame::Json {
                event: "response.output_text.delta",
                payload: json!({
                    "type": "response.output_text.delta",
                    "response_id": response_id,
                    "delta": delta,
                }),
            });
        }) as StreamDeltaHandler
    });

    // Single prompt invocation. The codex CLI backend runs its own internal
    // agent loop with --full-auto, handling multiple tool calls and verification.
    // No Ralph loop needed — codex already loops until the task is complete.
    let prompt_result = if state.config.turn_timeout_ms == 0 {
        agent
            .prompt_with_stream(&translated.prompt, stream_handler)
            .await
    } else {
        match tokio::time::timeout(
            Duration::from_millis(state.config.turn_timeout_ms),
            agent.prompt_with_stream(&translated.prompt, stream_handler),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                return Err(OpenResponsesApiError::timeout(
                    "response generation timed out before completion",
                ));
            }
        }
    };
    let post_prompt_cost = agent.cost_snapshot();
    persist_session_usage_delta(&mut session_runtime, &pre_prompt_cost, &post_prompt_cost)
        .map_err(|error| {
            OpenResponsesApiError::internal(format!(
                "failed to persist gateway session usage summary: {error}"
            ))
        })?;

    let new_messages = prompt_result.map_err(|error| {
        OpenResponsesApiError::gateway_failure(format!("gateway runtime failed: {error}"))
    })?;
    persist_messages(&mut session_runtime, &new_messages).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to persist gateway session messages: {error}"
        ))
    })?;

    let output_text = collect_assistant_reply(&agent.messages()[start_index..]);
    let usage = usage
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("prompt usage lock is poisoned"))?
        .clone();
    let finish_reason = finish_reason
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("prompt finish reason lock is poisoned"))?
        .clone()
        .unwrap_or_else(|| "stop".to_string());

    let mut ignored = BTreeSet::new();
    for field in translated.ignored_fields {
        if !field.trim().is_empty() {
            ignored.insert(field);
        }
    }

    let response = OpenResponsesResponse {
        id: response_id,
        object: "response",
        created,
        status: "completed",
        finish_reason,
        model: state.config.model.clone(),
        output: vec![OpenResponsesOutputItem {
            id: state.next_output_message_id(),
            kind: "message",
            role: "assistant",
            content: vec![OpenResponsesOutputTextItem {
                kind: "output_text",
                text: output_text.clone(),
            }],
        }],
        output_text,
        usage: OpenResponsesUsage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
        },
        ignored_fields: ignored.into_iter().collect(),
    };

    Ok(OpenResponsesExecutionResult { response })
}

const AUTO_GATEWAY_SKILL_ACTION_TOKENS: &[&str] = &[
    "build",
    "create",
    "develop",
    "edit",
    "fix",
    "implement",
    "make",
    "prototype",
    "scaffold",
    "ship",
    "write",
];

const AUTO_GATEWAY_SKILL_STOPWORDS: &[&str] = &[
    "a",
    "an",
    "and",
    "are",
    "can",
    "completely",
    "entire",
    "explain",
    "for",
    "how",
    "i",
    "in",
    "into",
    "it",
    "its",
    "of",
    "please",
    "process",
    "the",
    "to",
    "use",
    "using",
    "want",
    "with",
    "would",
    "you",
];

fn select_gateway_auto_skills(
    catalog: &[GatewayOpenResponsesSkillPrompt],
    explicit_names: &[String],
    prompt: &str,
) -> Vec<GatewayOpenResponsesSkillPrompt> {
    let explicit_keys = explicit_names
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let prompt_tokens = tokenize_gateway_skill_prompt(prompt);
    let actionable = prompt_tokens
        .iter()
        .any(|token| AUTO_GATEWAY_SKILL_ACTION_TOKENS.contains(&token.as_str()));
    if !actionable {
        return Vec::new();
    }

    catalog
        .iter()
        .filter(|skill| !explicit_keys.contains(&skill.name.to_ascii_lowercase()))
        .filter(|skill| score_gateway_skill_relevance(skill, &prompt_tokens) >= 2)
        .cloned()
        .collect::<Vec<_>>()
}

fn tokenize_gateway_skill_prompt(prompt: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut seen = HashSet::new();

    for raw in prompt
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-')
        .filter(|token| !token.trim().is_empty())
    {
        let normalized = normalize_gateway_skill_token(raw);
        if normalized.is_empty()
            || AUTO_GATEWAY_SKILL_STOPWORDS.contains(&normalized.as_str())
            || normalized.len() < 3
        {
            continue;
        }
        if seen.insert(normalized.clone()) {
            tokens.push(normalized);
        }
    }

    tokens
}

fn normalize_gateway_skill_token(token: &str) -> String {
    let lowered = token.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "phaserjs" => "phaser".to_string(),
        "games" | "gameplay" => "game".to_string(),
        "playable" => "play".to_string(),
        _ => lowered,
    }
}

fn score_gateway_skill_relevance(
    skill: &GatewayOpenResponsesSkillPrompt,
    prompt_tokens: &[String],
) -> usize {
    let haystack = format!(
        "{} {} {}",
        skill.name.to_ascii_lowercase(),
        skill.description.to_ascii_lowercase(),
        skill.content.to_ascii_lowercase()
    );

    prompt_tokens.iter().fold(0, |score, token| {
        if !haystack.contains(token) {
            return score;
        }
        score
            + match token.as_str() {
                "phaser" => 3,
                "game" => 1,
                _ => 1,
            }
    })
}

fn augment_gateway_system_prompt(base: &str, skills: &[GatewayOpenResponsesSkillPrompt]) -> String {
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
