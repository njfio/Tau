//! Helpers for rendering/persisting runtime output and event snapshots.
//!
//! These functions form a presentation boundary: they should not mutate agent
//! behavior, but they do preserve operator-visible traces and message summaries
//! used by troubleshooting and post-run audits.

use std::io::Write;

use anyhow::Result;
use serde_json::{json, Value};
use tau_agent_core::{extract_skip_response_reason, AgentEvent};
use tau_ai::{Message, MessageRole};
use tau_session::SessionRuntime;

/// Summarize one chat message for compact operator-facing logs.
pub fn summarize_message(message: &Message) -> String {
    let text = message.text_content().replace('\n', " ");
    if text.trim().is_empty() {
        return format!(
            "{:?} (tool_calls={})",
            message.role,
            message.tool_calls().len()
        );
    }

    let max = 60;
    if text.chars().count() <= max {
        text
    } else {
        let summary = text.chars().take(max).collect::<String>();
        format!("{summary}...")
    }
}

pub fn persist_messages(
    session_runtime: &mut Option<SessionRuntime>,
    new_messages: &[Message],
) -> Result<()> {
    let Some(runtime) = session_runtime.as_mut() else {
        return Ok(());
    };

    runtime.active_head = runtime
        .store
        .append_messages(runtime.active_head, new_messages)?;
    Ok(())
}

pub fn print_assistant_messages(
    messages: &[Message],
    stream_output: bool,
    _stream_delay_ms: u64,
    suppress_first_streamed_text: bool,
) {
    if extract_skip_response_reason(messages).is_some() {
        return;
    }
    let mut suppressed_once = false;
    for message in messages {
        if message.role != MessageRole::Assistant {
            continue;
        }

        let text = message.text_content();
        if !text.trim().is_empty() {
            if stream_output && suppress_first_streamed_text && !suppressed_once {
                suppressed_once = true;
                println!("\n");
                continue;
            }
            println!();
            if stream_output {
                let mut stdout = std::io::stdout();
                for chunk in stream_text_chunks(&text) {
                    print!("{chunk}");
                    let _ = stdout.flush();
                }
                println!("\n");
            } else {
                println!("{text}\n");
            }
            continue;
        }

        let tool_calls = message.tool_calls();
        if !tool_calls.is_empty() {
            println!(
                "\n[assistant requested {} tool call(s)]\n",
                tool_calls.len()
            );
        }
    }
}

pub fn stream_text_chunks(text: &str) -> Vec<&str> {
    text.split_inclusive(char::is_whitespace).collect()
}

/// Convert one agent event into deterministic JSON for logs and snapshots.
pub fn event_to_json(event: &AgentEvent) -> Value {
    lifecycle_event_json(event)
        .or_else(|| turn_event_json(event))
        .or_else(|| tool_event_json(event))
        .or_else(|| safety_event_json(event))
        .unwrap_or_else(unknown_event_json)
}

fn lifecycle_event_json(event: &AgentEvent) -> Option<Value> {
    match event {
        AgentEvent::AgentStart => Some(json!({ "type": "agent_start" })),
        AgentEvent::AgentEnd { new_messages } => {
            Some(json!({ "type": "agent_end", "new_messages": new_messages }))
        }
        AgentEvent::MessageAdded { message } => Some(message_added_json(message)),
        _ => None,
    }
}

fn turn_event_json(event: &AgentEvent) -> Option<Value> {
    match event {
        AgentEvent::TurnStart { turn } => Some(turn_start_json(*turn)),
        AgentEvent::TurnEnd {
            turn,
            tool_results,
            request_duration_ms,
            usage,
            finish_reason,
        } => Some(turn_end_json(
            *turn,
            *tool_results,
            *request_duration_ms,
            usage,
            finish_reason,
        )),
        AgentEvent::ReplanTriggered { turn, reason } => Some(json!({
            "type": "replan_triggered",
            "turn": turn,
            "reason": reason,
        })),
        AgentEvent::CostUpdated {
            turn,
            turn_cost_usd,
            cumulative_cost_usd,
            budget_usd,
        } => Some(json!({
            "type": "cost_updated",
            "turn": turn,
            "turn_cost_usd": turn_cost_usd,
            "cumulative_cost_usd": cumulative_cost_usd,
            "budget_usd": budget_usd,
        })),
        AgentEvent::CostBudgetAlert {
            turn,
            threshold_percent,
            cumulative_cost_usd,
            budget_usd,
        } => Some(json!({
            "type": "cost_budget_alert",
            "turn": turn,
            "threshold_percent": threshold_percent,
            "cumulative_cost_usd": cumulative_cost_usd,
            "budget_usd": budget_usd,
        })),
        _ => None,
    }
}

fn tool_event_json(event: &AgentEvent) -> Option<Value> {
    match event {
        AgentEvent::ToolExecutionStart {
            tool_call_id,
            tool_name,
            arguments,
        } => Some(tool_execution_start_json(
            tool_call_id,
            tool_name,
            arguments,
        )),
        AgentEvent::ToolExecutionEnd {
            tool_call_id,
            tool_name,
            result,
        } => Some(tool_execution_end_json(tool_call_id, tool_name, result)),
        _ => None,
    }
}

fn safety_event_json(event: &AgentEvent) -> Option<Value> {
    match event {
        AgentEvent::SafetyPolicyApplied {
            stage,
            mode,
            blocked,
            matched_rules,
            reason_codes,
        } => Some(json!({
            "type": "safety_policy_applied",
            "stage": stage.as_str(),
            "mode": mode,
            "blocked": blocked,
            "matched_rules": matched_rules,
            "reason_codes": reason_codes,
        })),
        _ => None,
    }
}

fn unknown_event_json() -> Value {
    json!({ "type": "unknown_event" })
}

fn message_added_json(message: &Message) -> Value {
    json!({
        "type": "message_added",
        "role": format!("{:?}", message.role).to_lowercase(),
        "text": message.text_content(),
        "tool_calls": message.tool_calls().len(),
    })
}

fn turn_start_json(turn: usize) -> Value {
    json!({
        "type": "turn_start",
        "turn": turn,
        "operator_state": turn_operator_state("in_progress", "model", turn),
    })
}

fn turn_end_json(
    turn: usize,
    tool_results: usize,
    request_duration_ms: u64,
    usage: &tau_ai::ChatUsage,
    finish_reason: &Option<String>,
) -> Value {
    json!({
        "type": "turn_end",
        "turn": turn,
        "tool_results": tool_results,
        "request_duration_ms": request_duration_ms,
        "usage": usage,
        "finish_reason": finish_reason,
        "operator_state": turn_completed_operator_state(turn, tool_results, finish_reason),
    })
}

fn tool_execution_start_json(tool_call_id: &str, tool_name: &str, arguments: &Value) -> Value {
    json!({
        "type": "tool_execution_start",
        "tool_call_id": tool_call_id,
        "tool_name": tool_name,
        "arguments": arguments,
        "operator_state": tool_operator_state("in_progress", tool_call_id, tool_name),
    })
}

fn tool_execution_end_json(
    tool_call_id: &str,
    tool_name: &str,
    result: &tau_agent_core::ToolExecutionResult,
) -> Value {
    json!({
        "type": "tool_execution_end",
        "tool_call_id": tool_call_id,
        "tool_name": tool_name,
        "is_error": result.is_error,
        "content": result.content,
        "operator_state": tool_operator_state(tool_completion_status(result.is_error), tool_call_id, tool_name),
    })
}

fn turn_operator_state(status: &str, phase: &str, turn: usize) -> Value {
    json!({
        "entity": "turn",
        "status": status,
        "phase": phase,
        "turn": turn,
    })
}

fn turn_completed_operator_state(
    turn: usize,
    tool_results: usize,
    finish_reason: &Option<String>,
) -> Value {
    let mut state = turn_operator_state("completed", "done", turn);
    let value = state
        .as_object_mut()
        .expect("turn operator state should always serialize to an object");
    value.insert("tool_results".to_string(), json!(tool_results));
    value.insert("finish_reason".to_string(), json!(finish_reason));
    state
}

fn tool_operator_state(status: &str, tool_call_id: &str, tool_name: &str) -> Value {
    json!({
        "entity": "tool",
        "status": status,
        "tool_call_id": tool_call_id,
        "tool_name": tool_name,
    })
}

fn tool_completion_status(is_error: bool) -> &'static str {
    if is_error {
        "failed"
    } else {
        "completed"
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{event_to_json, print_assistant_messages, stream_text_chunks, summarize_message};
    use tau_agent_core::{AgentEvent, SafetyMode, SafetyStage, ToolExecutionResult};
    use tau_ai::{ContentBlock, Message, ToolCall};

    #[test]
    fn unit_stream_text_chunks_preserve_whitespace_boundaries() {
        let chunks = stream_text_chunks("hello world\nnext");
        assert_eq!(chunks, vec!["hello ", "world\n", "next"]);
    }

    #[test]
    fn regression_stream_text_chunks_handles_empty_and_single_word() {
        assert!(stream_text_chunks("").is_empty());
        assert_eq!(stream_text_chunks("token"), vec!["token"]);
    }

    #[test]
    fn unit_summarize_message_truncates_long_text_and_reports_tool_calls_for_empty_text() {
        let short = Message::assistant_text("short text");
        assert_eq!(summarize_message(&short), "short text");

        let long = Message::assistant_text("a".repeat(80));
        assert!(summarize_message(&long).ends_with("..."));

        let tool_call = Message::assistant_blocks(vec![ContentBlock::tool_call(ToolCall {
            id: "call-1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({ "path": "README.md" }),
        })]);
        assert_eq!(summarize_message(&tool_call), "Assistant (tool_calls=1)");
    }

    #[test]
    fn unit_event_to_json_maps_tool_execution_end_shape() {
        let event = AgentEvent::ToolExecutionEnd {
            tool_call_id: "call-1".to_string(),
            tool_name: "write".to_string(),
            result: ToolExecutionResult::ok(serde_json::json!({ "ok": true })),
        };
        let value = event_to_json(&event);
        assert_eq!(value["type"], "tool_execution_end");
        assert_eq!(value["tool_call_id"], "call-1");
        assert_eq!(value["tool_name"], "write");
        assert_eq!(value["is_error"], false);
        assert_eq!(value["content"]["ok"], true);
    }

    #[test]
    fn red_spec_3581_event_to_json_emits_operator_state_for_turn_and_tool_events() {
        let turn_value = event_to_json(&AgentEvent::TurnStart { turn: 7 });
        assert_eq!(turn_value["operator_state"]["entity"], "turn");
        assert_eq!(turn_value["operator_state"]["phase"], "model");
        assert_eq!(turn_value["operator_state"]["status"], "in_progress");

        let tool_value = event_to_json(&AgentEvent::ToolExecutionStart {
            tool_call_id: "call-7".to_string(),
            tool_name: "http".to_string(),
            arguments: serde_json::json!({ "url": "https://example.com" }),
        });
        assert_eq!(tool_value["operator_state"]["entity"], "tool");
        assert_eq!(tool_value["operator_state"]["status"], "in_progress");
        assert_eq!(tool_value["operator_state"]["tool_name"], "http");
        assert_eq!(tool_value["operator_state"]["tool_call_id"], "call-7");
    }

    #[test]
    fn unit_event_to_json_maps_replan_triggered_shape() {
        let event = AgentEvent::ReplanTriggered {
            turn: 2,
            reason: "tool failure".to_string(),
        };
        let value = event_to_json(&event);
        assert_eq!(value["type"], "replan_triggered");
        assert_eq!(value["turn"], 2);
        assert_eq!(value["reason"], "tool failure");
    }

    #[test]
    fn unit_event_to_json_maps_cost_budget_alert_shape() {
        let event = AgentEvent::CostBudgetAlert {
            turn: 3,
            threshold_percent: 80,
            cumulative_cost_usd: 1.25,
            budget_usd: 1.5,
        };
        let value = event_to_json(&event);
        assert_eq!(value["type"], "cost_budget_alert");
        assert_eq!(value["turn"], 3);
        assert_eq!(value["threshold_percent"], 80);
        assert_eq!(value["cumulative_cost_usd"], 1.25);
        assert_eq!(value["budget_usd"], 1.5);
    }

    #[test]
    fn unit_event_to_json_maps_safety_policy_applied_shape() {
        let event = AgentEvent::SafetyPolicyApplied {
            stage: SafetyStage::ToolOutput,
            mode: SafetyMode::Block,
            blocked: true,
            matched_rules: vec!["literal.ignore_previous_instructions".to_string()],
            reason_codes: vec!["prompt_injection.ignore_instructions".to_string()],
        };
        let value = event_to_json(&event);
        assert_eq!(value["type"], "safety_policy_applied");
        assert_eq!(value["stage"], "tool_output");
        assert_eq!(value["mode"], "block");
        assert_eq!(value["blocked"], true);
        assert_eq!(
            value["reason_codes"][0],
            "prompt_injection.ignore_instructions"
        );
    }

    #[test]
    fn unit_print_assistant_messages_stream_fallback_avoids_blocking_delay() {
        let started = Instant::now();
        print_assistant_messages(
            &[Message::assistant_text("fallback stream render")],
            true,
            300,
            false,
        );
        assert!(
            started.elapsed() < Duration::from_millis(350),
            "fallback render should not sleep per chunk in sync path"
        );
    }
}
