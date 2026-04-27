//! OpenResponses execution handler.

use super::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use tau_ai::Message;
use tau_memory::action_history::ActionHistoryStore;

use super::learning_runtime::{
    load_gateway_action_history_store, save_gateway_action_history_store,
};

const GATEWAY_TOOL_SUMMARY_MAX_CHARS: usize = 240;

#[derive(Debug, Clone)]
struct GatewayPendingToolExecution {
    tool_name: String,
    arguments: Value,
    started_unix_ms: u64,
}

#[derive(Debug, Clone)]
struct GatewayObservedToolExecution {
    tool_name: String,
    arguments: Value,
    output_summary: String,
    success: bool,
    latency_ms: u64,
    timestamp_ms: u64,
}

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
                    "id": response_id.clone(),
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
            max_turns: state.config.max_turns,
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
    register_gateway_completion_tool(&mut agent);

    let usage = Arc::new(Mutex::new(OpenResponsesUsageSummary::default()));
    let finish_reason = Arc::new(Mutex::new(None::<String>));
    let tool_execution_count = Arc::new(AtomicUsize::new(0));
    let tool_execution_starts = Arc::new(Mutex::new(
        HashMap::<String, GatewayPendingToolExecution>::new(),
    ));
    let tool_execution_traces = Arc::new(Mutex::new(Vec::<GatewayObservedToolExecution>::new()));
    let event_response_id = response_id.clone();
    let event_stream_sender = stream_sender.clone();
    agent.subscribe({
        let usage = usage.clone();
        let finish_reason = finish_reason.clone();
        let tool_execution_count = tool_execution_count.clone();
        let tool_execution_starts = tool_execution_starts.clone();
        let tool_execution_traces = tool_execution_traces.clone();
        let event_response_id = event_response_id.clone();
        let event_stream_sender = event_stream_sender.clone();
        move |event| match event {
            AgentEvent::TurnEnd {
                usage: turn_usage,
                finish_reason: turn_finish_reason,
                ..
            } => {
                if let Ok(mut guard) = usage.lock() {
                    guard.input_tokens = guard.input_tokens.saturating_add(turn_usage.input_tokens);
                    guard.output_tokens =
                        guard.output_tokens.saturating_add(turn_usage.output_tokens);
                    guard.total_tokens = guard.total_tokens.saturating_add(turn_usage.total_tokens);
                }
                if let Ok(mut guard) = finish_reason.lock() {
                    *guard = turn_finish_reason.clone();
                }
            }
            AgentEvent::ToolExecutionStart {
                tool_call_id,
                tool_name,
                arguments,
                ..
            } => {
                tool_execution_count.fetch_add(1, Ordering::Relaxed);
                if let Ok(mut guard) = tool_execution_starts.lock() {
                    guard.insert(
                        tool_call_id.clone(),
                        GatewayPendingToolExecution {
                            tool_name: tool_name.clone(),
                            arguments: arguments.clone(),
                            started_unix_ms: current_unix_timestamp_ms(),
                        },
                    );
                }
                if tool_name != GATEWAY_COMPLETE_TASK_TOOL_NAME {
                    if let Some(sender) = &event_stream_sender {
                        let _ = sender.send(SseFrame::Json {
                            event: "response.tool_execution.started",
                            payload: json!({
                                "type": "response.tool_execution.started",
                                "response_id": event_response_id.as_str(),
                                "tool_call_id": tool_call_id,
                                "tool_name": tool_name,
                                "arguments": arguments,
                            }),
                        });
                    }
                }
            }
            AgentEvent::ToolExecutionEnd {
                tool_call_id,
                tool_name,
                result,
            } => {
                let now_unix_ms = current_unix_timestamp_ms();
                let pending = tool_execution_starts
                    .lock()
                    .ok()
                    .and_then(|mut guard| guard.remove(tool_call_id.as_str()));
                let arguments = pending
                    .as_ref()
                    .map(|entry| entry.arguments.clone())
                    .unwrap_or_else(|| json!({}));
                let latency_ms = pending
                    .as_ref()
                    .map(|entry| now_unix_ms.saturating_sub(entry.started_unix_ms))
                    .unwrap_or(0);
                if let Ok(mut guard) = tool_execution_traces.lock() {
                    guard.push(GatewayObservedToolExecution {
                        tool_name: tool_name.clone(),
                        arguments,
                        output_summary: summarize_gateway_tool_text(result.as_text().as_str()),
                        success: !result.is_error,
                        latency_ms,
                        timestamp_ms: now_unix_ms,
                    });
                }
                if tool_name != GATEWAY_COMPLETE_TASK_TOOL_NAME {
                    if let Some(sender) = &event_stream_sender {
                        let _ = sender.send(SseFrame::Json {
                            event: "response.tool_execution.completed",
                            payload: json!({
                                "type": "response.tool_execution.completed",
                                "response_id": event_response_id.as_str(),
                                "tool_call_id": tool_call_id,
                                "tool_name": tool_name,
                                "success": !result.is_error,
                                "timed_out": false,
                                "latency_ms": latency_ms,
                                "timestamp_ms": now_unix_ms,
                            }),
                        });
                    }
                }
            }
            _ => {}
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

    let start_index = agent.messages().len();
    let pre_prompt_cost = agent.cost_snapshot();
    let prompt_tokens = tokenize_gateway_skill_prompt(&translated.prompt);
    let delegated = state.config.delegated_tool_execution;
    let requires_tool_evidence = !delegated && gateway_prompt_tokens_request_action(&prompt_tokens);
    let requires_mutation_evidence =
        !delegated && gateway_prompt_tokens_request_mutation(&prompt_tokens);
    let requires_validation_evidence =
        !delegated && gateway_prompt_tokens_request_validation(&prompt_tokens);
    let mission_path =
        gateway_mission_state_path(&state.config.state_dir, translated.mission_id.as_str());
    let mut mission_state = GatewayMissionState::load_or_create(
        &mission_path,
        translated.mission_id.as_str(),
        translated.session_key.as_str(),
        response_id.as_str(),
        translated.prompt.as_str(),
        current_unix_timestamp_ms(),
    )?;
    save_gateway_mission_state(&mission_path, &mission_state)?;
    let buffered_stream_output = if requires_tool_evidence && stream_sender.is_some() {
        Some(Arc::new(Mutex::new(String::new())))
    } else {
        None
    };
    let mut retry_attempt = 0usize;
    let mut next_prompt = translated.prompt.clone();
    let mut terminal_success_verifier = None::<GatewayMissionVerifierBundle>;
    let mut terminal_completion_signal = None::<GatewayMissionCompletionSignalRecord>;
    let mut terminal_output_override = None::<String>;
    let mut action_history_store = load_gateway_action_history_store(&state.config.state_dir)?;
    let prompt_result: Result<(), OpenResponsesApiError> = loop {
        refresh_gateway_learning_system_prompt(
            &mut agent,
            &state,
            &action_history_store,
            &auto_selected,
            requires_tool_evidence,
        )?;
        let attempt_number = retry_attempt.saturating_add(1);
        let attempt_started_unix_ms = current_unix_timestamp_ms();
        reset_buffered_gateway_output(buffered_stream_output.as_ref())?;
        let attempt_start_index = agent.messages().len();
        let attempt_request_payload = build_gateway_attempt_request_payload(
            response_id.as_str(),
            translated.mission_id.as_str(),
            translated.session_key.as_str(),
            attempt_number,
            next_prompt.as_str(),
            agent.messages(),
        );
        let tool_trace_start_index = gateway_tool_trace_len(&tool_execution_traces)?;
        let tool_execution_count_before = tool_execution_count.load(Ordering::Relaxed);
        let stream_handler = build_gateway_stream_handler(
            stream_sender.as_ref(),
            response_id.as_str(),
            buffered_stream_output.as_ref(),
        );
        let attempt_result = if state.config.turn_timeout_ms == 0 {
            agent.prompt_with_stream(&next_prompt, stream_handler).await
        } else {
            match tokio::time::timeout(
                Duration::from_millis(state.config.turn_timeout_ms),
                agent.prompt_with_stream(&next_prompt, stream_handler),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    finalize_pending_gateway_tool_executions(
                        &tool_execution_starts,
                        &tool_execution_traces,
                        stream_sender.as_ref(),
                        response_id.as_str(),
                        true,
                        "tool execution timed out before completion",
                    )?;
                    persist_gateway_attempt_tool_history(
                        &mut action_history_store,
                        translated.session_key.as_str(),
                        translated.mission_id.as_str(),
                        attempt_number,
                        &tool_execution_traces,
                        tool_trace_start_index,
                    )?;
                    let finished_unix_ms = current_unix_timestamp_ms();
                    let tool_execution_delta = tool_execution_count
                        .load(Ordering::Relaxed)
                        .saturating_sub(tool_execution_count_before);
                    let retry_exhausted = retry_attempt >= ACTION_TOOL_EVIDENCE_MAX_RETRIES;
                    let verifier_traces = snapshot_gateway_verifier_traces(&tool_execution_traces)?;
                    let verifier = if tool_execution_delta == 0 {
                        build_gateway_runtime_failure_verifier_bundle(
                            "gateway_timeout",
                            "response generation timed out before completion",
                        )
                    } else {
                        build_gateway_verifier_bundle(
                            requires_tool_evidence,
                            requires_mutation_evidence,
                            requires_validation_evidence,
                            verifier_traces.as_slice(),
                            retry_exhausted,
                        )
                    };
                    let attempt_response_payload = build_gateway_attempt_response_payload(
                        &agent.messages()[attempt_start_index..],
                        &tool_execution_traces,
                        tool_trace_start_index,
                        "timeout",
                        Some("response generation timed out before completion"),
                    )?;
                    mission_state.record_iteration(GatewayMissionIterationInput {
                        attempt: attempt_number,
                        prompt: next_prompt.as_str(),
                        assistant_summary: "",
                        tool_execution_count: tool_execution_delta,
                        request_payload: attempt_request_payload.clone(),
                        response_payload: attempt_response_payload,
                        verifier: verifier.clone(),
                        completion: None,
                        started_unix_ms: attempt_started_unix_ms,
                        finished_unix_ms,
                    });
                    if verifier.overall.status == GatewayMissionVerifierStatus::Continue
                        && !retry_exhausted
                    {
                        save_gateway_mission_state(&mission_path, &mission_state)?;
                        strip_failed_action_attempt_assistant_messages(
                            &mut agent,
                            attempt_start_index,
                        );
                        retry_attempt = retry_attempt.saturating_add(1);
                        next_prompt = build_gateway_action_retry_prompt(retry_attempt, &verifier);
                        continue;
                    }
                    mission_state.mark_blocked(verifier.overall, None, "", finished_unix_ms);
                    save_gateway_mission_state(&mission_path, &mission_state)?;
                    break Err(OpenResponsesApiError::timeout(
                        "response generation timed out before completion",
                    ));
                }
            }
        };
        if let Err(error) = attempt_result {
            finalize_pending_gateway_tool_executions(
                &tool_execution_starts,
                &tool_execution_traces,
                stream_sender.as_ref(),
                response_id.as_str(),
                false,
                "tool execution aborted before completion",
            )?;
            persist_gateway_attempt_tool_history(
                &mut action_history_store,
                translated.session_key.as_str(),
                translated.mission_id.as_str(),
                attempt_number,
                &tool_execution_traces,
                tool_trace_start_index,
            )?;
            let finished_unix_ms = current_unix_timestamp_ms();
            let message = format!("gateway runtime failed: {error}");
            let verifier = build_gateway_runtime_failure_verifier_bundle(
                "gateway_runtime_error",
                message.as_str(),
            );
            let assistant_summary =
                collect_assistant_reply(&agent.messages()[attempt_start_index..]);
            let attempt_response_payload = build_gateway_attempt_response_payload(
                &agent.messages()[attempt_start_index..],
                &tool_execution_traces,
                tool_trace_start_index,
                "error",
                Some(message.as_str()),
            )?;
            mission_state.record_iteration(GatewayMissionIterationInput {
                attempt: attempt_number,
                prompt: next_prompt.as_str(),
                assistant_summary: assistant_summary.as_str(),
                tool_execution_count: 0,
                request_payload: attempt_request_payload.clone(),
                response_payload: attempt_response_payload,
                verifier: verifier.clone(),
                completion: None,
                started_unix_ms: attempt_started_unix_ms,
                finished_unix_ms,
            });
            mission_state.mark_blocked(
                verifier.overall,
                None,
                assistant_summary.as_str(),
                finished_unix_ms,
            );
            save_gateway_mission_state(&mission_path, &mission_state)?;
            break Err(OpenResponsesApiError::gateway_failure(message));
        }

        let tool_execution_delta = tool_execution_count
            .load(Ordering::Relaxed)
            .saturating_sub(tool_execution_count_before);
        persist_gateway_attempt_tool_history(
            &mut action_history_store,
            translated.session_key.as_str(),
            translated.mission_id.as_str(),
            attempt_number,
            &tool_execution_traces,
            tool_trace_start_index,
        )?;
        let finished_unix_ms = current_unix_timestamp_ms();
        let assistant_summary = collect_assistant_reply(&agent.messages()[attempt_start_index..]);
        let retry_exhausted = retry_attempt >= ACTION_TOOL_EVIDENCE_MAX_RETRIES;
        let verifier_traces = snapshot_gateway_verifier_traces(&tool_execution_traces)?;
        let completion_signal = extract_gateway_completion_signal(&verifier_traces);
        let verifier = build_gateway_verifier_bundle(
            requires_tool_evidence,
            requires_mutation_evidence,
            requires_validation_evidence,
            verifier_traces.as_slice(),
            retry_exhausted,
        );
        let attempt_response_payload = build_gateway_attempt_response_payload(
            &agent.messages()[attempt_start_index..],
            &tool_execution_traces,
            tool_trace_start_index,
            "completed",
            None,
        )?;
        mission_state.record_iteration(GatewayMissionIterationInput {
            attempt: attempt_number,
            prompt: next_prompt.as_str(),
            assistant_summary: assistant_summary.as_str(),
            tool_execution_count: tool_execution_delta,
            request_payload: attempt_request_payload.clone(),
            response_payload: attempt_response_payload,
            verifier: verifier.clone(),
            completion: completion_signal.clone(),
            started_unix_ms: attempt_started_unix_ms,
            finished_unix_ms,
        });
        if let Some(completion) = completion_signal.clone() {
            match completion.status {
                GatewayMissionCompletionStatus::Partial => {
                    mission_state.mark_checkpointed(
                        verifier.overall.clone(),
                        completion.clone(),
                        completion.summary.as_str(),
                        finished_unix_ms,
                    );
                    save_gateway_mission_state(&mission_path, &mission_state)?;
                    terminal_output_override = Some(completion.summary.clone());
                    terminal_completion_signal = Some(completion);
                    flush_buffered_gateway_output(
                        stream_sender.as_ref(),
                        response_id.as_str(),
                        buffered_stream_output.as_ref(),
                    )?;
                    break Ok(());
                }
                GatewayMissionCompletionStatus::Blocked => {
                    mission_state.mark_blocked(
                        verifier.overall.clone(),
                        Some(completion.clone()),
                        completion.summary.as_str(),
                        finished_unix_ms,
                    );
                    save_gateway_mission_state(&mission_path, &mission_state)?;
                    terminal_output_override = Some(completion.summary.clone());
                    terminal_completion_signal = Some(completion);
                    flush_buffered_gateway_output(
                        stream_sender.as_ref(),
                        response_id.as_str(),
                        buffered_stream_output.as_ref(),
                    )?;
                    break Ok(());
                }
                GatewayMissionCompletionStatus::Success => {
                    if verifier.overall.status == GatewayMissionVerifierStatus::Passed {
                        save_gateway_mission_state(&mission_path, &mission_state)?;
                        terminal_success_verifier = Some(verifier);
                        terminal_output_override = Some(completion.summary.clone());
                        terminal_completion_signal = Some(completion);
                        flush_buffered_gateway_output(
                            stream_sender.as_ref(),
                            response_id.as_str(),
                            buffered_stream_output.as_ref(),
                        )?;
                        break Ok(());
                    }
                }
            }
        }
        match verifier.overall.status {
            GatewayMissionVerifierStatus::Passed => {
                save_gateway_mission_state(&mission_path, &mission_state)?;
                terminal_success_verifier = Some(verifier);
                flush_buffered_gateway_output(
                    stream_sender.as_ref(),
                    response_id.as_str(),
                    buffered_stream_output.as_ref(),
                )?;
                break Ok(());
            }
            GatewayMissionVerifierStatus::Failed => {
                mission_state.mark_blocked(
                    verifier.overall.clone(),
                    None,
                    assistant_summary.as_str(),
                    finished_unix_ms,
                );
                save_gateway_mission_state(&mission_path, &mission_state)?;
                break Err(OpenResponsesApiError::gateway_failure(
                    verifier.overall.message.clone(),
                ));
            }
            GatewayMissionVerifierStatus::Continue => {
                save_gateway_mission_state(&mission_path, &mission_state)?;
            }
        }
        strip_failed_action_attempt_assistant_messages(&mut agent, attempt_start_index);
        retry_attempt = retry_attempt.saturating_add(1);
        next_prompt = build_gateway_action_retry_prompt(retry_attempt, &verifier);
    };
    save_gateway_action_history_store(&state.config.state_dir, &action_history_store)?;
    let post_prompt_cost = agent.cost_snapshot();
    persist_session_usage_delta(&mut session_runtime, &pre_prompt_cost, &post_prompt_cost)
        .map_err(|error| {
            OpenResponsesApiError::internal(format!(
                "failed to persist gateway session usage summary: {error}"
            ))
        })?;

    prompt_result?;
    let new_messages = agent.messages()[start_index..].to_vec();
    persist_messages(&mut session_runtime, &new_messages).map_err(|error| {
        OpenResponsesApiError::internal(format!(
            "failed to persist gateway session messages: {error}"
        ))
    })?;

    let mut output_text = collect_assistant_reply(&agent.messages()[start_index..]);
    if let Some(summary) = terminal_output_override.clone() {
        output_text = summary;
    }
    if let Some(verifier) = terminal_success_verifier {
        mission_state.mark_completed(
            verifier.overall,
            terminal_completion_signal,
            output_text.as_str(),
            current_unix_timestamp_ms(),
        );
        save_gateway_mission_state(&mission_path, &mission_state)?;
    }
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

const ACTION_TOOL_EVIDENCE_MAX_RETRIES: usize = 2;

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
    if !gateway_prompt_tokens_request_action(&prompt_tokens) {
        return Vec::new();
    }

    catalog
        .iter()
        .filter(|skill| !explicit_keys.contains(&skill.name.to_ascii_lowercase()))
        .filter(|skill| score_gateway_skill_relevance(skill, &prompt_tokens) >= 2)
        .cloned()
        .collect::<Vec<_>>()
}

fn build_gateway_stream_handler(
    stream_sender: Option<&mpsc::UnboundedSender<SseFrame>>,
    response_id: &str,
    buffered_output: Option<&Arc<Mutex<String>>>,
) -> Option<StreamDeltaHandler> {
    match (stream_sender, buffered_output) {
        (Some(_sender), Some(buffer)) => {
            let buffer = Arc::clone(buffer);
            let _response_id = response_id.to_string();
            Some(Arc::new(move |delta: String| {
                if delta.is_empty() {
                    return;
                }
                if let Ok(mut guard) = buffer.lock() {
                    guard.push_str(&delta);
                }
            }) as StreamDeltaHandler)
        }
        (Some(sender), None) => {
            let sender = sender.clone();
            let response_id = response_id.to_string();
            Some(Arc::new(move |delta: String| {
                if delta.is_empty() {
                    return;
                }
                let _ = sender.send(SseFrame::Json {
                    event: "response.output_text.delta",
                    payload: json!({
                        "type": "response.output_text.delta",
                        "response_id": response_id.as_str(),
                        "delta": delta,
                    }),
                });
            }) as StreamDeltaHandler)
        }
        (None, _) => None,
    }
}

fn reset_buffered_gateway_output(
    buffered_output: Option<&Arc<Mutex<String>>>,
) -> Result<(), OpenResponsesApiError> {
    if let Some(buffer) = buffered_output {
        let mut guard = buffer.lock().map_err(|_| {
            OpenResponsesApiError::internal("gateway stream buffer lock is poisoned")
        })?;
        guard.clear();
    }
    Ok(())
}

fn flush_buffered_gateway_output(
    stream_sender: Option<&mpsc::UnboundedSender<SseFrame>>,
    response_id: &str,
    buffered_output: Option<&Arc<Mutex<String>>>,
) -> Result<(), OpenResponsesApiError> {
    let Some(sender) = stream_sender else {
        return Ok(());
    };
    let Some(buffer) = buffered_output else {
        return Ok(());
    };
    let delta = {
        let guard = buffer.lock().map_err(|_| {
            OpenResponsesApiError::internal("gateway stream buffer lock is poisoned")
        })?;
        guard.clone()
    };
    if delta.is_empty() {
        return Ok(());
    }
    let _ = sender.send(SseFrame::Json {
        event: "response.output_text.delta",
        payload: json!({
            "type": "response.output_text.delta",
            "response_id": response_id,
            "delta": delta,
        }),
    });
    Ok(())
}

fn strip_failed_action_attempt_assistant_messages(agent: &mut Agent, attempt_start_index: usize) {
    let retained_messages = agent.messages()[..attempt_start_index]
        .iter()
        .cloned()
        .chain(
            agent.messages()[attempt_start_index..]
                .iter()
                .filter(|message| message.role != MessageRole::Assistant)
                .cloned(),
        )
        .collect::<Vec<_>>();
    agent.replace_messages(retained_messages);
}

fn build_gateway_action_retry_prompt(
    retry_attempt: usize,
    verifier: &GatewayMissionVerifierBundle,
) -> String {
    format!(
        "{} Retry attempt: {retry_attempt}. Do not reply with another promise, plan, or status update unless you have first satisfied the active verifier requirements or hit a concrete runtime blocker.",
        build_gateway_retry_feedback(verifier)
    )
}

fn refresh_gateway_learning_system_prompt(
    agent: &mut Agent,
    state: &GatewayOpenResponsesServerState,
    action_history_store: &ActionHistoryStore,
    auto_selected: &[GatewayOpenResponsesSkillPrompt],
    completion_tool_active: bool,
) -> Result<(), OpenResponsesApiError> {
    let mut effective_system_prompt = state.resolved_system_prompt();
    let learning_bulletin =
        render_gateway_learning_bulletin(action_history_store, GATEWAY_ACTION_HISTORY_LOOKBACK);
    if !learning_bulletin.trim().is_empty() {
        if effective_system_prompt.trim().is_empty() {
            effective_system_prompt = learning_bulletin;
        } else {
            effective_system_prompt = format!(
                "{}\n\n{}",
                effective_system_prompt.trim_end(),
                learning_bulletin.trim()
            );
        }
    }
    if !auto_selected.is_empty() {
        effective_system_prompt =
            augment_gateway_system_prompt(&effective_system_prompt, auto_selected);
    }
    if completion_tool_active {
        if effective_system_prompt.trim().is_empty() {
            effective_system_prompt = render_gateway_completion_guidance().to_string();
        } else {
            effective_system_prompt = format!(
                "{}\n\n{}",
                effective_system_prompt.trim_end(),
                render_gateway_completion_guidance()
            );
        }
    }
    let _ = agent.replace_system_prompt(effective_system_prompt);
    Ok(())
}

fn gateway_tool_trace_len(
    traces: &Arc<Mutex<Vec<GatewayObservedToolExecution>>>,
) -> Result<usize, OpenResponsesApiError> {
    traces
        .lock()
        .map(|guard| guard.len())
        .map_err(|_| OpenResponsesApiError::internal("gateway tool trace lock is poisoned"))
}

fn build_gateway_attempt_request_payload(
    response_id: &str,
    mission_id: &str,
    session_key: &str,
    attempt_number: usize,
    prompt: &str,
    messages_before: &[Message],
) -> Value {
    json!({
        "response_id": response_id,
        "mission_id": mission_id,
        "session_id": session_key,
        "attempt": attempt_number,
        "prompt": prompt,
        "messages_before": serialize_gateway_messages(messages_before),
    })
}

fn build_gateway_attempt_response_payload(
    messages_after: &[Message],
    traces: &Arc<Mutex<Vec<GatewayObservedToolExecution>>>,
    trace_start_index: usize,
    status: &str,
    error_message: Option<&str>,
) -> Result<Value, OpenResponsesApiError> {
    Ok(json!({
        "status": status,
        "error": error_message,
        "messages": serialize_gateway_messages(messages_after),
        "tool_executions": snapshot_gateway_attempt_tool_payloads(traces, trace_start_index)?,
    }))
}

fn serialize_gateway_messages(messages: &[Message]) -> Value {
    serde_json::to_value(messages).unwrap_or_else(|error| {
        json!({
            "serialization_error": error.to_string(),
        })
    })
}

fn snapshot_gateway_attempt_tool_payloads(
    traces: &Arc<Mutex<Vec<GatewayObservedToolExecution>>>,
    trace_start_index: usize,
) -> Result<Vec<Value>, OpenResponsesApiError> {
    let payloads = traces
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("gateway tool trace lock is poisoned"))?
        .iter()
        .skip(trace_start_index)
        .map(|trace| {
            json!({
                "tool_name": trace.tool_name,
                "arguments": trace.arguments,
                "output_summary": trace.output_summary,
                "success": trace.success,
                "latency_ms": trace.latency_ms,
                "timestamp_ms": trace.timestamp_ms,
            })
        })
        .collect::<Vec<_>>();
    Ok(payloads)
}

fn snapshot_gateway_verifier_traces(
    traces: &Arc<Mutex<Vec<GatewayObservedToolExecution>>>,
) -> Result<Vec<GatewayVerifierToolTrace>, OpenResponsesApiError> {
    let traces = traces
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("gateway tool trace lock is poisoned"))?
        .iter()
        .cloned()
        .map(|trace| GatewayVerifierToolTrace {
            tool_name: trace.tool_name,
            arguments: trace.arguments,
            success: trace.success,
        })
        .collect::<Vec<_>>();
    Ok(traces)
}

fn finalize_pending_gateway_tool_executions(
    pending_tools: &Arc<Mutex<HashMap<String, GatewayPendingToolExecution>>>,
    traces: &Arc<Mutex<Vec<GatewayObservedToolExecution>>>,
    stream_sender: Option<&mpsc::UnboundedSender<SseFrame>>,
    response_id: &str,
    timed_out: bool,
    failure_message: &str,
) -> Result<(), OpenResponsesApiError> {
    let pending = pending_tools
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("gateway tool start lock is poisoned"))?
        .drain()
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return Ok(());
    }

    let now_unix_ms = current_unix_timestamp_ms();
    let output_summary = summarize_gateway_tool_text(failure_message);
    let mut trace_guard = traces
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("gateway tool trace lock is poisoned"))?;
    for (tool_call_id, pending) in pending {
        let latency_ms = now_unix_ms.saturating_sub(pending.started_unix_ms);
        trace_guard.push(GatewayObservedToolExecution {
            tool_name: pending.tool_name.clone(),
            arguments: pending.arguments.clone(),
            output_summary: output_summary.clone(),
            success: false,
            latency_ms,
            timestamp_ms: now_unix_ms,
        });
        if pending.tool_name != GATEWAY_COMPLETE_TASK_TOOL_NAME {
            if let Some(sender) = stream_sender {
                let _ = sender.send(SseFrame::Json {
                    event: "response.tool_execution.completed",
                    payload: json!({
                        "type": "response.tool_execution.completed",
                        "response_id": response_id,
                        "tool_call_id": tool_call_id,
                        "tool_name": pending.tool_name,
                        "arguments": pending.arguments,
                        "success": false,
                        "timed_out": timed_out,
                        "latency_ms": latency_ms,
                        "timestamp_ms": now_unix_ms,
                    }),
                });
            }
        }
    }
    Ok(())
}

fn persist_gateway_attempt_tool_history(
    action_history_store: &mut ActionHistoryStore,
    session_key: &str,
    mission_id: &str,
    attempt_number: usize,
    traces: &Arc<Mutex<Vec<GatewayObservedToolExecution>>>,
    trace_start_index: usize,
) -> Result<(), OpenResponsesApiError> {
    let records = traces
        .lock()
        .map_err(|_| OpenResponsesApiError::internal("gateway tool trace lock is poisoned"))?
        .iter()
        .skip(trace_start_index)
        .filter(|trace| trace.tool_name != GATEWAY_COMPLETE_TASK_TOOL_NAME)
        .cloned()
        .map(|trace| GatewayActionHistoryToolRecord {
            session_key: session_key.to_string(),
            mission_id: mission_id.to_string(),
            turn: attempt_number,
            tool_name: trace.tool_name,
            input_summary: summarize_gateway_tool_value(&trace.arguments),
            output_summary: trace.output_summary,
            success: trace.success,
            latency_ms: trace.latency_ms,
            timestamp_ms: trace.timestamp_ms,
        })
        .collect::<Vec<_>>();
    append_gateway_action_history_records(action_history_store, &records);
    Ok(())
}

fn summarize_gateway_tool_value(value: &Value) -> String {
    summarize_gateway_tool_text(serde_json::to_string(value).unwrap_or_default().as_str())
}

fn summarize_gateway_tool_text(raw: &str) -> String {
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized
        .chars()
        .take(GATEWAY_TOOL_SUMMARY_MAX_CHARS)
        .collect()
}

fn gateway_prompt_tokens_request_action(prompt_tokens: &[String]) -> bool {
    prompt_tokens
        .iter()
        .any(|token| AUTO_GATEWAY_SKILL_ACTION_TOKENS.contains(&token.as_str()))
}

fn gateway_prompt_tokens_request_mutation(prompt_tokens: &[String]) -> bool {
    prompt_tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "create"
                | "fix"
                | "write"
                | "edit"
                | "update"
                | "modify"
                | "generate"
                | "implement"
                | "scaffold"
                | "make"
                | "change"
                | "delete"
                | "remove"
                | "rename"
        )
    })
}

fn gateway_prompt_tokens_request_validation(prompt_tokens: &[String]) -> bool {
    prompt_tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "test"
                | "tested"
                | "validate"
                | "validated"
                | "verify"
                | "verified"
                | "check"
                | "checked"
                | "playable"
                | "lint"
                | "run"
                | "runnable"
                | "compile"
                | "compiled"
        )
    })
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
