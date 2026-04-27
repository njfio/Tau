use std::path::{Path, PathBuf};

pub(super) fn write_multi_channel_runtime_fixture(root: &Path, with_connectors: bool) -> PathBuf {
    let multi_channel_root = root.join(".tau").join("multi-channel");
    std::fs::create_dir_all(&multi_channel_root).expect("create multi-channel root");
    std::fs::write(
        multi_channel_root.join("state.json"),
        r#"{
  "schema_version": 1,
  "processed_event_keys": ["telegram:tg-1", "discord:dc-1", "telegram:tg-2"],
  "health": {
    "updated_unix_ms": 981,
    "cycle_duration_ms": 14,
    "queue_depth": 2,
    "active_runs": 0,
    "failure_streak": 1,
    "last_cycle_discovered": 3,
    "last_cycle_processed": 3,
    "last_cycle_completed": 2,
    "last_cycle_failed": 1,
    "last_cycle_duplicates": 0
  }
}
"#,
    )
    .expect("write multi-channel state");
    std::fs::write(
            multi_channel_root.join("runtime-events.jsonl"),
            r#"{"reason_codes":["events_applied","connector_retry"],"health_reason":"connector retry in progress"}
invalid-json-line
{"reason_codes":["connector_retry"],"health_reason":"connector retry in progress"}
"#,
        )
        .expect("write runtime events");
    if with_connectors {
        std::fs::write(
            multi_channel_root.join("live-connectors-state.json"),
            r#"{
  "schema_version": 1,
  "processed_event_keys": ["telegram:tg-1"],
  "channels": {
    "telegram": {
      "mode": "polling",
      "liveness": "open",
      "events_ingested": 6,
      "duplicates_skipped": 2,
      "retry_attempts": 3,
      "auth_failures": 1,
      "parse_failures": 0,
      "provider_failures": 2,
      "consecutive_failures": 2,
      "retry_budget_remaining": 0,
      "breaker_state": "open",
      "breaker_open_until_unix_ms": 4000,
      "breaker_last_open_reason": "provider_unavailable",
      "breaker_open_count": 1,
      "last_error_code": "provider_unavailable"
    }
  }
}
"#,
        )
        .expect("write connectors state");
    }
    multi_channel_root
}

pub(super) fn write_dashboard_runtime_fixture(root: &Path) -> PathBuf {
    let dashboard_root = root.join(".tau").join("dashboard");
    std::fs::create_dir_all(&dashboard_root).expect("create dashboard root");
    std::fs::write(
        dashboard_root.join("state.json"),
        r#"{
  "schema_version": 1,
  "processed_case_keys": ["snapshot:s1", "control:c1"],
  "widget_views": [
    {
      "widget_id": "health-summary",
      "kind": "health_summary",
      "title": "Runtime Health",
      "query_key": "runtime.health",
      "refresh_interval_ms": 3000,
      "last_case_key": "snapshot:s1",
      "updated_unix_ms": 810
    },
    {
      "widget_id": "run-timeline",
      "kind": "run_timeline",
      "title": "Run Timeline",
      "query_key": "runtime.timeline",
      "refresh_interval_ms": 7000,
      "last_case_key": "snapshot:s1",
      "updated_unix_ms": 811
    }
  ],
  "control_audit": [{"event_key":"dashboard-control:resume:c1"}],
  "health": {
    "updated_unix_ms": 812,
    "cycle_duration_ms": 21,
    "queue_depth": 1,
    "active_runs": 0,
    "failure_streak": 0,
    "last_cycle_discovered": 2,
    "last_cycle_processed": 2,
    "last_cycle_completed": 2,
    "last_cycle_failed": 0,
    "last_cycle_duplicates": 0
  }
}
"#,
    )
    .expect("write dashboard state");
    std::fs::write(
        dashboard_root.join("runtime-events.jsonl"),
        r#"{"timestamp_unix_ms":810,"health_state":"healthy","health_reason":"no recent transport failures observed","reason_codes":["widget_views_updated"],"discovered_cases":2,"queued_cases":2,"backlog_cases":0,"applied_cases":2,"failed_cases":0}
invalid-json-line
{"timestamp_unix_ms":811,"health_state":"healthy","health_reason":"no recent transport failures observed","reason_codes":["widget_views_updated","control_actions_applied"],"discovered_cases":2,"queued_cases":2,"backlog_cases":0,"applied_cases":2,"failed_cases":0}
"#,
    )
    .expect("write dashboard events");
    dashboard_root
}

pub(super) fn write_dashboard_runtime_fixture_nominal(root: &Path) -> PathBuf {
    let dashboard_root = root.join(".tau").join("dashboard");
    std::fs::create_dir_all(&dashboard_root).expect("create dashboard root");
    std::fs::write(
        dashboard_root.join("state.json"),
        r#"{
  "schema_version": 1,
  "processed_case_keys": ["snapshot:s1"],
  "widget_views": [
    {
      "widget_id": "health-summary",
      "kind": "health_summary",
      "title": "Runtime Health",
      "query_key": "runtime.health",
      "refresh_interval_ms": 3000,
      "last_case_key": "snapshot:s1",
      "updated_unix_ms": 900
    }
  ],
  "control_audit": [],
  "health": {
    "updated_unix_ms": 901,
    "cycle_duration_ms": 20,
    "queue_depth": 0,
    "active_runs": 0,
    "failure_streak": 0,
    "last_cycle_discovered": 1,
    "last_cycle_processed": 1,
    "last_cycle_completed": 1,
    "last_cycle_failed": 0,
    "last_cycle_duplicates": 0
  }
}
"#,
    )
    .expect("write nominal dashboard state");
    std::fs::write(
        dashboard_root.join("runtime-events.jsonl"),
        r#"{"timestamp_unix_ms":900,"health_state":"healthy","health_reason":"dashboard runtime health is nominal","reason_codes":[],"discovered_cases":1,"queued_cases":1,"backlog_cases":0,"applied_cases":1,"failed_cases":0}
"#,
    )
    .expect("write nominal dashboard events");
    dashboard_root
}

pub(super) fn write_dashboard_control_state_fixture(root: &Path) -> PathBuf {
    let dashboard_root = root.join(".tau").join("dashboard");
    std::fs::create_dir_all(&dashboard_root).expect("create dashboard root");
    std::fs::write(
        dashboard_root.join("control-state.json"),
        r#"{
  "schema_version": 1,
  "mode": "paused",
  "updated_unix_ms": 90210,
  "last_action": {
    "schema_version": 1,
    "request_id": "dashboard-action-90210",
    "action": "pause",
    "actor": "ops-user",
    "reason": "maintenance",
    "status": "accepted",
    "timestamp_unix_ms": 90210,
    "control_mode": "paused"
  }
}
"#,
    )
    .expect("write dashboard control state");
    dashboard_root
}

pub(super) fn write_training_runtime_fixture(root: &Path, failed: usize) -> PathBuf {
    let training_root = root.join(".tau").join("training");
    std::fs::create_dir_all(&training_root).expect("create training root");
    std::fs::write(
        training_root.join("status.json"),
        format!(
            r#"{{
  "schema_version": 1,
  "updated_unix_ms": 900,
  "run_state": "completed",
  "model_ref": "openai/gpt-5.2",
  "store_path": ".tau/training/store.sqlite",
  "total_rollouts": 4,
  "succeeded": {succeeded},
  "failed": {failed},
  "cancelled": 0
}}
"#,
            succeeded = 4usize.saturating_sub(failed),
            failed = failed
        ),
    )
    .expect("write training status");
    training_root
}

pub(super) fn write_training_rollouts_fixture(root: &Path) -> PathBuf {
    let training_root = root.join(".tau").join("training");
    std::fs::create_dir_all(&training_root).expect("create training root");
    std::fs::write(
        training_root.join("rollouts.jsonl"),
        r#"{"rollout_id":"r-104","status":"succeeded","mode":"optimize","steps":12,"reward":0.9,"duration_ms":3000,"updated_unix_ms":1400}
invalid-rollout-line
{"rollout_id":"r-103","status":"cancelled","mode":"validate","steps":3,"reward":0.1,"duration_ms":1100,"updated_unix_ms":1300}
{"rollout_id":"r-102","status":"failed","mode":"optimize","steps":8,"reward":-0.3,"duration_ms":2500,"updated_unix_ms":1200}
"#,
    )
    .expect("write training rollouts");
    training_root
}

pub(super) fn write_gateway_audit_fixture(root: &Path) -> (PathBuf, PathBuf) {
    let dashboard_root = root.join(".tau").join("dashboard");
    std::fs::create_dir_all(&dashboard_root).expect("create dashboard root for audit fixture");
    std::fs::write(
        dashboard_root.join("actions-audit.jsonl"),
        r#"{"schema_version":1,"request_id":"dashboard-action-1","action":"pause","actor":"ops-user-1","reason":"maintenance","status":"accepted","timestamp_unix_ms":1000,"control_mode":"paused"}
invalid-dashboard-line
{"schema_version":1,"request_id":"dashboard-action-2","action":"resume","actor":"ops-user-2","reason":"maintenance-complete","status":"accepted","timestamp_unix_ms":2000,"control_mode":"running"}
"#,
    )
    .expect("write dashboard actions audit fixture");

    let telemetry_root = root.join(".tau").join("gateway").join("openresponses");
    std::fs::create_dir_all(&telemetry_root).expect("create telemetry root for audit fixture");
    std::fs::write(
        telemetry_root.join("ui-telemetry.jsonl"),
        r#"{"timestamp_unix_ms":1500,"view":"dashboard","action":"refresh","reason_code":"ui_refresh","session_key":"default","principal":"ops-user-1","metadata":{"surface":"webchat"}}
{"timestamp_unix_ms":2500,"view":"memory","action":"search","reason_code":"memory_search_requested","session_key":"s-memory","principal":"ops-user-2","metadata":{"query":"ArcSwap"}}
invalid-telemetry-line
"#,
    )
    .expect("write ui telemetry fixture");

    (
        dashboard_root.join("actions-audit.jsonl"),
        telemetry_root.join("ui-telemetry.jsonl"),
    )
}

pub(super) fn write_tools_telemetry_fixture(root: &Path) -> PathBuf {
    let telemetry_root = root.join(".tau").join("gateway").join("openresponses");
    std::fs::create_dir_all(&telemetry_root).expect("create telemetry root");
    std::fs::write(
        telemetry_root.join("ui-telemetry.jsonl"),
        r#"{"timestamp_unix_ms":1000,"view":"tools","action":"invoke","reason_code":"tool_invoked","session_key":"s-tools-1","principal":"ops-user","metadata":{"tool_name":"bash"}}
{"timestamp_unix_ms":1100,"view":"tools","action":"invoke","reason_code":"tool_invoked","session_key":"s-tools-1","principal":"ops-user","metadata":{"tool_name":"memory_search"}}
invalid-tools-telemetry-line
{"timestamp_unix_ms":1200,"view":"tools","action":"invoke","reason_code":"tool_invoked","session_key":"s-tools-2","principal":"ops-user","metadata":{"tool_name":"bash"}}
{"timestamp_unix_ms":1300,"view":"memory","action":"search","reason_code":"memory_search_requested","session_key":"s-memory","principal":"ops-user","metadata":{"tool_name":"memory_search"}}
"#,
    )
    .expect("write tools telemetry fixture");
    telemetry_root.join("ui-telemetry.jsonl")
}

pub(super) fn write_events_runtime_fixture(root: &Path) -> PathBuf {
    let events_root = root.join(".tau").join("events");
    std::fs::create_dir_all(&events_root).expect("create events root");
    std::fs::write(
        events_root.join("deploy.json"),
        r#"{
  "id": "deploy-routine",
  "channel": "slack/C123",
  "prompt": "Post deployment status.",
  "schedule": { "type": "immediate" },
  "enabled": true,
  "created_unix_ms": 1700000000000
}
"#,
    )
    .expect("write events definition");
    std::fs::write(
        events_root.join("state.json"),
        r#"{
  "schema_version": 1,
  "periodic_last_run_unix_ms": {},
  "debounce_last_seen_unix_ms": {},
  "signature_replay_last_seen_unix_ms": {},
  "recent_executions": [
    {
      "timestamp_unix_ms": 1700000005000,
      "event_id": "deploy-routine",
      "channel": "slack/C123",
      "schedule": "immediate",
      "outcome": "executed",
      "reason_code": "event_executed"
    }
  ]
}
"#,
    )
    .expect("write events state");
    events_root
}
