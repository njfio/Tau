use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::channel_store::{ChannelContextEntry, ChannelLogEntry, ChannelStore};
use crate::multi_channel_contract::{
    event_contract_key, load_multi_channel_contract_fixture, MultiChannelContractFixture,
    MultiChannelEventKind, MultiChannelInboundEvent,
};
use crate::{current_unix_timestamp_ms, write_text_atomic};

const MULTI_CHANNEL_RUNTIME_STATE_SCHEMA_VERSION: u32 = 1;

fn multi_channel_runtime_state_schema_version() -> u32 {
    MULTI_CHANNEL_RUNTIME_STATE_SCHEMA_VERSION
}

#[derive(Debug, Clone)]
pub(crate) struct MultiChannelRuntimeConfig {
    pub(crate) fixture_path: PathBuf,
    pub(crate) state_dir: PathBuf,
    pub(crate) queue_limit: usize,
    pub(crate) processed_event_cap: usize,
    pub(crate) retry_max_attempts: usize,
    pub(crate) retry_base_delay_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MultiChannelRuntimeSummary {
    pub(crate) discovered_events: usize,
    pub(crate) queued_events: usize,
    pub(crate) completed_events: usize,
    pub(crate) duplicate_skips: usize,
    pub(crate) transient_failures: usize,
    pub(crate) retry_attempts: usize,
    pub(crate) failed_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MultiChannelRuntimeState {
    #[serde(default = "multi_channel_runtime_state_schema_version")]
    schema_version: u32,
    #[serde(default)]
    processed_event_keys: Vec<String>,
}

impl Default for MultiChannelRuntimeState {
    fn default() -> Self {
        Self {
            schema_version: MULTI_CHANNEL_RUNTIME_STATE_SCHEMA_VERSION,
            processed_event_keys: Vec::new(),
        }
    }
}

pub(crate) async fn run_multi_channel_contract_runner(
    config: MultiChannelRuntimeConfig,
) -> Result<()> {
    let fixture = load_multi_channel_contract_fixture(&config.fixture_path)?;
    let mut runtime = MultiChannelRuntime::new(config)?;
    let summary = runtime.run_once(&fixture).await?;
    println!(
        "multi-channel runner summary: discovered={} queued={} completed={} duplicate_skips={} retries={} transient_failures={} failed={}",
        summary.discovered_events,
        summary.queued_events,
        summary.completed_events,
        summary.duplicate_skips,
        summary.retry_attempts,
        summary.transient_failures,
        summary.failed_events
    );
    Ok(())
}

struct MultiChannelRuntime {
    config: MultiChannelRuntimeConfig,
    state: MultiChannelRuntimeState,
    processed_event_keys: HashSet<String>,
}

impl MultiChannelRuntime {
    fn new(config: MultiChannelRuntimeConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.state_dir)
            .with_context(|| format!("failed to create {}", config.state_dir.display()))?;
        let mut state = load_multi_channel_runtime_state(&config.state_dir.join("state.json"))?;
        state.processed_event_keys =
            normalize_processed_keys(&state.processed_event_keys, config.processed_event_cap);
        let processed_event_keys = state.processed_event_keys.iter().cloned().collect();
        Ok(Self {
            config,
            state,
            processed_event_keys,
        })
    }

    fn state_path(&self) -> PathBuf {
        self.config.state_dir.join("state.json")
    }

    async fn run_once(
        &mut self,
        fixture: &MultiChannelContractFixture,
    ) -> Result<MultiChannelRuntimeSummary> {
        let mut summary = MultiChannelRuntimeSummary {
            discovered_events: fixture.events.len(),
            ..MultiChannelRuntimeSummary::default()
        };

        let mut queued_events = fixture.events.clone();
        queued_events.sort_by(|left, right| {
            left.timestamp_ms
                .cmp(&right.timestamp_ms)
                .then_with(|| event_contract_key(left).cmp(&event_contract_key(right)))
        });
        queued_events.truncate(self.config.queue_limit);
        summary.queued_events = queued_events.len();

        for event in queued_events {
            let event_key = event_contract_key(&event);
            if self.processed_event_keys.contains(&event_key) {
                summary.duplicate_skips = summary.duplicate_skips.saturating_add(1);
                continue;
            }

            let simulated_transient_failures = simulated_transient_failures(&event);
            let mut attempt = 1usize;
            loop {
                if attempt <= simulated_transient_failures {
                    summary.transient_failures = summary.transient_failures.saturating_add(1);
                    if attempt >= self.config.retry_max_attempts {
                        summary.failed_events = summary.failed_events.saturating_add(1);
                        break;
                    }
                    summary.retry_attempts = summary.retry_attempts.saturating_add(1);
                    apply_retry_delay(self.config.retry_base_delay_ms, attempt).await;
                    attempt = attempt.saturating_add(1);
                    continue;
                }

                match self.persist_event(&event, &event_key) {
                    Ok(()) => {
                        self.record_processed_event(&event_key);
                        summary.completed_events = summary.completed_events.saturating_add(1);
                        break;
                    }
                    Err(error) => {
                        if attempt >= self.config.retry_max_attempts {
                            eprintln!(
                                "multi-channel runner event failed: key={} transport={} error={error}",
                                event_key,
                                event.transport.as_str()
                            );
                            summary.failed_events = summary.failed_events.saturating_add(1);
                            break;
                        }
                        summary.transient_failures = summary.transient_failures.saturating_add(1);
                        summary.retry_attempts = summary.retry_attempts.saturating_add(1);
                        apply_retry_delay(self.config.retry_base_delay_ms, attempt).await;
                        attempt = attempt.saturating_add(1);
                    }
                }
            }
        }

        save_multi_channel_runtime_state(&self.state_path(), &self.state)?;
        Ok(summary)
    }

    fn persist_event(&self, event: &MultiChannelInboundEvent, event_key: &str) -> Result<()> {
        let store = ChannelStore::open(
            &self.config.state_dir.join("channel-store"),
            event.transport.as_str(),
            &event.conversation_id,
        )?;
        let timestamp_unix_ms = current_unix_timestamp_ms();

        store.append_log_entry(&ChannelLogEntry {
            timestamp_unix_ms,
            direction: "inbound".to_string(),
            event_key: Some(event_key.to_string()),
            source: event.transport.as_str().to_string(),
            payload: serde_json::to_value(event).context("serialize inbound event payload")?,
        })?;

        if !event.text.trim().is_empty() {
            store.append_context_entry(&ChannelContextEntry {
                timestamp_unix_ms,
                role: "user".to_string(),
                text: event.text.trim().to_string(),
            })?;
        }

        let response_text = render_response(event);
        store.append_log_entry(&ChannelLogEntry {
            timestamp_unix_ms: current_unix_timestamp_ms(),
            direction: "outbound".to_string(),
            event_key: Some(event_key.to_string()),
            source: "tau-multi-channel-runner".to_string(),
            payload: json!({
                "response": response_text,
                "event_key": event_key,
                "transport": event.transport.as_str(),
            }),
        })?;
        store.append_context_entry(&ChannelContextEntry {
            timestamp_unix_ms: current_unix_timestamp_ms(),
            role: "assistant".to_string(),
            text: response_text,
        })?;

        Ok(())
    }

    fn record_processed_event(&mut self, event_key: &str) {
        if self.processed_event_keys.contains(event_key) {
            return;
        }
        self.state.processed_event_keys.push(event_key.to_string());
        self.processed_event_keys.insert(event_key.to_string());
        if self.state.processed_event_keys.len() > self.config.processed_event_cap {
            let overflow = self
                .state
                .processed_event_keys
                .len()
                .saturating_sub(self.config.processed_event_cap);
            let removed = self.state.processed_event_keys.drain(0..overflow);
            for key in removed {
                self.processed_event_keys.remove(&key);
            }
        }
    }
}

fn render_response(event: &MultiChannelInboundEvent) -> String {
    let transport = event.transport.as_str();
    let event_id = event.event_id.trim();
    if matches!(event.event_kind, MultiChannelEventKind::Command)
        || event.text.trim().starts_with('/')
    {
        return format!(
            "command acknowledged: transport={} event_id={} conversation={}",
            transport, event_id, event.conversation_id
        );
    }
    format!(
        "message processed: transport={} event_id={} text_chars={}",
        transport,
        event_id,
        event.text.chars().count()
    )
}

fn normalize_processed_keys(raw: &[String], cap: usize) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for key in raw {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            continue;
        }
        let owned = trimmed.to_string();
        if seen.insert(owned.clone()) {
            normalized.push(owned);
        }
    }
    if cap == 0 {
        return Vec::new();
    }
    if normalized.len() > cap {
        normalized.drain(0..normalized.len().saturating_sub(cap));
    }
    normalized
}

fn simulated_transient_failures(event: &MultiChannelInboundEvent) -> usize {
    event
        .metadata
        .get("simulate_transient_failures")
        .and_then(|value| value.as_u64())
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
}

fn retry_delay_ms(base_delay_ms: u64, attempt: usize) -> u64 {
    if base_delay_ms == 0 {
        return 0;
    }
    let exponent = attempt.saturating_sub(1).min(10) as u32;
    base_delay_ms.saturating_mul(1_u64 << exponent)
}

async fn apply_retry_delay(base_delay_ms: u64, attempt: usize) {
    let delay_ms = retry_delay_ms(base_delay_ms, attempt);
    if delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
}

fn load_multi_channel_runtime_state(path: &Path) -> Result<MultiChannelRuntimeState> {
    if !path.exists() {
        return Ok(MultiChannelRuntimeState::default());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed = match serde_json::from_str::<MultiChannelRuntimeState>(&raw) {
        Ok(state) => state,
        Err(error) => {
            eprintln!(
                "multi-channel runner: failed to parse state file {} ({error}); starting fresh",
                path.display()
            );
            return Ok(MultiChannelRuntimeState::default());
        }
    };
    if parsed.schema_version != MULTI_CHANNEL_RUNTIME_STATE_SCHEMA_VERSION {
        eprintln!(
            "multi-channel runner: unsupported state schema {} in {}; starting fresh",
            parsed.schema_version,
            path.display()
        );
        return Ok(MultiChannelRuntimeState::default());
    }
    Ok(parsed)
}

fn save_multi_channel_runtime_state(path: &Path, state: &MultiChannelRuntimeState) -> Result<()> {
    let payload = serde_json::to_string_pretty(state).context("serialize multi-channel state")?;
    write_text_atomic(path, &payload).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    use super::{
        load_multi_channel_runtime_state, retry_delay_ms, MultiChannelRuntime,
        MultiChannelRuntimeConfig,
    };
    use crate::channel_store::ChannelStore;
    use crate::multi_channel_contract::{
        load_multi_channel_contract_fixture, parse_multi_channel_contract_fixture,
    };

    fn fixture_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("multi-channel-contract")
            .join(name)
    }

    fn build_config(root: &Path) -> MultiChannelRuntimeConfig {
        MultiChannelRuntimeConfig {
            fixture_path: fixture_path("baseline-three-channel.json"),
            state_dir: root.join(".tau/multi-channel"),
            queue_limit: 64,
            processed_event_cap: 10_000,
            retry_max_attempts: 3,
            retry_base_delay_ms: 0,
        }
    }

    #[test]
    fn unit_retry_delay_ms_scales_with_attempt_number() {
        assert_eq!(retry_delay_ms(0, 1), 0);
        assert_eq!(retry_delay_ms(10, 1), 10);
        assert_eq!(retry_delay_ms(10, 2), 20);
        assert_eq!(retry_delay_ms(10, 3), 40);
    }

    #[tokio::test]
    async fn functional_runner_processes_fixture_and_persists_channel_store_entries() {
        let temp = tempdir().expect("tempdir");
        let config = build_config(temp.path());
        let fixture =
            load_multi_channel_contract_fixture(&config.fixture_path).expect("fixture should load");
        let mut runtime = MultiChannelRuntime::new(config.clone()).expect("runtime");
        let summary = runtime.run_once(&fixture).await.expect("run once");

        assert_eq!(summary.discovered_events, 3);
        assert_eq!(summary.queued_events, 3);
        assert_eq!(summary.completed_events, 3);
        assert_eq!(summary.duplicate_skips, 0);
        assert_eq!(summary.failed_events, 0);

        for event in &fixture.events {
            let store = ChannelStore::open(
                &config.state_dir.join("channel-store"),
                event.transport.as_str(),
                &event.conversation_id,
            )
            .expect("open store");
            let logs = store.load_log_entries().expect("load logs");
            let context = store.load_context_entries().expect("load context");
            assert_eq!(logs.len(), 2);
            assert!(context.len() >= 2);
        }
    }

    #[tokio::test]
    async fn integration_runner_retries_transient_failure_then_recovers() {
        let temp = tempdir().expect("tempdir");
        let mut config = build_config(temp.path());
        config.retry_max_attempts = 4;
        let fixture_raw = r#"{
  "schema_version": 1,
  "name": "transient-retry",
  "events": [
    {
      "schema_version": 1,
      "transport": "telegram",
      "event_kind": "message",
      "event_id": "tg-transient-1",
      "conversation_id": "telegram-chat-transient",
      "actor_id": "telegram-user-1",
      "timestamp_ms": 1760100000000,
      "text": "hello",
      "metadata": { "simulate_transient_failures": 1 }
    }
  ]
}"#;
        let fixture = parse_multi_channel_contract_fixture(fixture_raw).expect("parse fixture");
        let mut runtime = MultiChannelRuntime::new(config).expect("runtime");
        let summary = runtime.run_once(&fixture).await.expect("run once");

        assert_eq!(summary.completed_events, 1);
        assert_eq!(summary.transient_failures, 1);
        assert_eq!(summary.retry_attempts, 1);
        assert_eq!(summary.failed_events, 0);
    }

    #[tokio::test]
    async fn integration_runner_respects_queue_limit_for_backpressure() {
        let temp = tempdir().expect("tempdir");
        let mut config = build_config(temp.path());
        config.queue_limit = 2;
        let fixture =
            load_multi_channel_contract_fixture(&config.fixture_path).expect("fixture should load");
        let mut runtime = MultiChannelRuntime::new(config.clone()).expect("runtime");
        let summary = runtime.run_once(&fixture).await.expect("run once");

        assert_eq!(summary.discovered_events, 3);
        assert_eq!(summary.queued_events, 2);
        assert_eq!(summary.completed_events, 2);
        let state = load_multi_channel_runtime_state(&config.state_dir.join("state.json"))
            .expect("load state");
        assert_eq!(state.processed_event_keys.len(), 2);
    }

    #[tokio::test]
    async fn regression_runner_skips_duplicate_events_from_persisted_state() {
        let temp = tempdir().expect("tempdir");
        let config = build_config(temp.path());
        let fixture =
            load_multi_channel_contract_fixture(&config.fixture_path).expect("fixture should load");

        let mut first_runtime = MultiChannelRuntime::new(config.clone()).expect("first runtime");
        let first_summary = first_runtime.run_once(&fixture).await.expect("first run");
        assert_eq!(first_summary.completed_events, 3);

        let mut second_runtime = MultiChannelRuntime::new(config).expect("second runtime");
        let second_summary = second_runtime.run_once(&fixture).await.expect("second run");
        assert_eq!(second_summary.completed_events, 0);
        assert_eq!(second_summary.duplicate_skips, 3);
    }
}
