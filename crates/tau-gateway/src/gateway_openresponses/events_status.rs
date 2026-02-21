use std::path::Path;

use serde::{Deserialize, Serialize};
use tau_core::current_unix_timestamp_ms;

const GATEWAY_EVENTS_INSPECT_QUEUE_LIMIT: usize = 64;
const GATEWAY_EVENTS_STALE_IMMEDIATE_MAX_AGE_SECONDS: u64 = 86_400;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct GatewayEventsStatusReport {
    state_present: bool,
    events_dir: String,
    state_path: String,
    health_state: String,
    rollout_gate: String,
    reason_code: String,
    health_reason: String,
    discovered_events: usize,
    enabled_events: usize,
    due_now_events: usize,
    queued_now_events: usize,
    not_due_events: usize,
    stale_immediate_events: usize,
    malformed_events: usize,
    due_eval_failed_events: usize,
    execution_history_entries: usize,
    executed_history_entries: usize,
    failed_history_entries: usize,
    skipped_history_entries: usize,
    last_execution_unix_ms: Option<u64>,
    last_execution_reason_code: Option<String>,
    diagnostics: Vec<String>,
}

impl Default for GatewayEventsStatusReport {
    fn default() -> Self {
        Self {
            state_present: false,
            events_dir: String::new(),
            state_path: String::new(),
            health_state: "unknown".to_string(),
            rollout_gate: "hold".to_string(),
            reason_code: "events_status_unavailable".to_string(),
            health_reason: "events scheduler status is unavailable".to_string(),
            discovered_events: 0,
            enabled_events: 0,
            due_now_events: 0,
            queued_now_events: 0,
            not_due_events: 0,
            stale_immediate_events: 0,
            malformed_events: 0,
            due_eval_failed_events: 0,
            execution_history_entries: 0,
            executed_history_entries: 0,
            failed_history_entries: 0,
            skipped_history_entries: 0,
            last_execution_unix_ms: None,
            last_execution_reason_code: None,
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GatewayEventDefinition {
    id: String,
    channel: String,
    schedule: GatewayEventSchedule,
    #[serde(default = "default_gateway_event_enabled")]
    enabled: bool,
    #[serde(default)]
    created_unix_ms: Option<u64>,
}

fn default_gateway_event_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GatewayEventSchedule {
    Immediate,
    At { at_unix_ms: u64 },
    Periodic { cron: String, timezone: String },
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GatewayEventsStateFile {
    #[serde(default)]
    recent_executions: Vec<GatewayEventExecutionRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct GatewayEventExecutionRecord {
    timestamp_unix_ms: u64,
    outcome: String,
    reason_code: String,
}

pub(super) fn collect_gateway_events_status_report(
    gateway_state_dir: &Path,
) -> GatewayEventsStatusReport {
    let tau_root = gateway_state_dir.parent().unwrap_or(gateway_state_dir);
    let events_dir = tau_root.join("events");
    let state_path = events_dir.join("state.json");
    let events_dir_exists = events_dir.is_dir();
    let state_present = state_path.is_file();

    if !events_dir_exists && !state_present {
        return GatewayEventsStatusReport {
            state_present: false,
            events_dir: events_dir.display().to_string(),
            state_path: state_path.display().to_string(),
            health_state: "healthy".to_string(),
            rollout_gate: "pass".to_string(),
            reason_code: "events_not_configured".to_string(),
            health_reason: "events scheduler is not configured".to_string(),
            diagnostics: vec![
                "create event definitions under events_dir to enable routine scheduling"
                    .to_string(),
            ],
            ..GatewayEventsStatusReport::default()
        };
    }

    let state = if state_present {
        match std::fs::read_to_string(&state_path) {
            Ok(payload) => match serde_json::from_str::<GatewayEventsStateFile>(&payload) {
                Ok(parsed) => Some(parsed),
                Err(error) => {
                    return GatewayEventsStatusReport {
                        state_present,
                        events_dir: events_dir.display().to_string(),
                        state_path: state_path.display().to_string(),
                        health_state: "failing".to_string(),
                        rollout_gate: "hold".to_string(),
                        reason_code: "events_state_parse_failed".to_string(),
                        health_reason: "failed to parse events state payload".to_string(),
                        diagnostics: vec![error.to_string()],
                        ..GatewayEventsStatusReport::default()
                    };
                }
            },
            Err(error) => {
                return GatewayEventsStatusReport {
                    state_present,
                    events_dir: events_dir.display().to_string(),
                    state_path: state_path.display().to_string(),
                    health_state: "failing".to_string(),
                    rollout_gate: "hold".to_string(),
                    reason_code: "events_state_read_failed".to_string(),
                    health_reason: "failed to read events state payload".to_string(),
                    diagnostics: vec![error.to_string()],
                    ..GatewayEventsStatusReport::default()
                };
            }
        }
    } else {
        None
    };

    let mut discovered_events = 0usize;
    let mut enabled_events = 0usize;
    let mut due_now_events = 0usize;
    let mut not_due_events = 0usize;
    let mut stale_immediate_events = 0usize;
    let mut malformed_events = 0usize;
    let due_eval_failed_events = 0usize;
    let now_unix_ms = current_unix_timestamp_ms();

    if events_dir_exists {
        let entries = match std::fs::read_dir(&events_dir) {
            Ok(entries) => entries,
            Err(error) => {
                return GatewayEventsStatusReport {
                    state_present,
                    events_dir: events_dir.display().to_string(),
                    state_path: state_path.display().to_string(),
                    health_state: "failing".to_string(),
                    rollout_gate: "hold".to_string(),
                    reason_code: "events_dir_read_failed".to_string(),
                    health_reason: "failed to read events definitions directory".to_string(),
                    diagnostics: vec![error.to_string()],
                    ..GatewayEventsStatusReport::default()
                };
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(value) => value,
                Err(_) => {
                    malformed_events = malformed_events.saturating_add(1);
                    continue;
                }
            };
            let path = entry.path();
            if path == state_path {
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let payload = match std::fs::read_to_string(&path) {
                Ok(payload) => payload,
                Err(_) => {
                    malformed_events = malformed_events.saturating_add(1);
                    continue;
                }
            };
            let definition = match serde_json::from_str::<GatewayEventDefinition>(&payload) {
                Ok(definition) => definition,
                Err(_) => {
                    malformed_events = malformed_events.saturating_add(1);
                    continue;
                }
            };
            let _ = (&definition.id, &definition.channel);
            discovered_events = discovered_events.saturating_add(1);
            if definition.enabled {
                enabled_events = enabled_events.saturating_add(1);
            } else {
                not_due_events = not_due_events.saturating_add(1);
                continue;
            }

            match definition.schedule {
                GatewayEventSchedule::Immediate => {
                    let created = definition.created_unix_ms.unwrap_or(now_unix_ms);
                    let max_age_ms =
                        GATEWAY_EVENTS_STALE_IMMEDIATE_MAX_AGE_SECONDS.saturating_mul(1_000);
                    if GATEWAY_EVENTS_STALE_IMMEDIATE_MAX_AGE_SECONDS > 0
                        && now_unix_ms.saturating_sub(created) > max_age_ms
                    {
                        stale_immediate_events = stale_immediate_events.saturating_add(1);
                    } else {
                        due_now_events = due_now_events.saturating_add(1);
                    }
                }
                GatewayEventSchedule::At { at_unix_ms } => {
                    if now_unix_ms >= at_unix_ms {
                        due_now_events = due_now_events.saturating_add(1);
                    } else {
                        not_due_events = not_due_events.saturating_add(1);
                    }
                }
                GatewayEventSchedule::Periodic { cron, timezone } => {
                    let _ = (cron, timezone);
                    not_due_events = not_due_events.saturating_add(1);
                }
            }
        }
    }

    let queued_now_events = due_now_events.min(GATEWAY_EVENTS_INSPECT_QUEUE_LIMIT.max(1));
    let executions = state
        .as_ref()
        .map(|value| value.recent_executions.clone())
        .unwrap_or_default();
    let execution_history_entries = executions.len();
    let executed_history_entries = executions
        .iter()
        .filter(|entry| entry.outcome == "executed")
        .count();
    let failed_history_entries = executions
        .iter()
        .filter(|entry| entry.outcome == "failed")
        .count();
    let skipped_history_entries = executions
        .iter()
        .filter(|entry| entry.outcome == "skipped")
        .count();
    let last_execution_unix_ms = executions.last().map(|entry| entry.timestamp_unix_ms);
    let last_execution_reason_code = executions.last().map(|entry| entry.reason_code.clone());

    let mut health_state = "healthy".to_string();
    let mut rollout_gate = "pass".to_string();
    let mut reason_code = "events_ready".to_string();
    let mut health_reason = "events scheduler diagnostics are healthy".to_string();
    let mut diagnostics = Vec::new();

    if discovered_events == 0 {
        reason_code = "events_none_discovered".to_string();
        health_reason = "events directory is configured but contains no definitions".to_string();
        diagnostics.push("add event definition files to enable scheduled routines".to_string());
    }
    if malformed_events > 0 {
        health_state = "degraded".to_string();
        rollout_gate = "hold".to_string();
        reason_code = "events_malformed_definitions".to_string();
        health_reason = format!(
            "events inspect found {} malformed definition files",
            malformed_events
        );
        diagnostics
            .push("run --events-validate to repair malformed event definition files".to_string());
    }
    if failed_history_entries > 0 {
        health_state = "degraded".to_string();
        rollout_gate = "hold".to_string();
        reason_code = "events_recent_failures".to_string();
        health_reason = format!(
            "events execution history includes {} failed runs",
            failed_history_entries
        );
        diagnostics.push(
            "inspect channel-store logs and recent execution history for failing routines"
                .to_string(),
        );
    }

    GatewayEventsStatusReport {
        state_present,
        events_dir: events_dir.display().to_string(),
        state_path: state_path.display().to_string(),
        health_state,
        rollout_gate,
        reason_code,
        health_reason,
        discovered_events,
        enabled_events,
        due_now_events,
        queued_now_events,
        not_due_events,
        stale_immediate_events,
        malformed_events,
        due_eval_failed_events,
        execution_history_entries,
        executed_history_entries,
        failed_history_entries,
        skipped_history_entries,
        last_execution_unix_ms,
        last_execution_reason_code,
        diagnostics,
    }
}
