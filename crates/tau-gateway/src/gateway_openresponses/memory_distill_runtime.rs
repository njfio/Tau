//! Background runtime that distills salient user statements from session logs into semantic memory.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tau_ai::MessageRole;
use tau_core::{current_unix_timestamp_ms, write_text_atomic};
use tau_memory::memory_contract::{MemoryEntry, MemoryScope};
use tau_memory::runtime::MemoryType;
use tau_session::{SessionEntry, SessionStore};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::cortex_runtime::record_cortex_observer_event;
use super::*;

const MEMORY_DISTILL_CHECKPOINT_SCHEMA_VERSION: u32 = 1;
const MAX_REASON_CODES: usize = 16;
const MAX_RECENT_WRITES: usize = 16;

#[derive(Debug)]
pub(super) struct MemoryDistillRuntimeHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl MemoryDistillRuntimeHandle {
    pub(super) fn disabled() -> Self {
        Self {
            shutdown_tx: None,
            task: None,
        }
    }

    pub(super) async fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

pub(super) fn start_memory_distill_runtime(
    state: Arc<GatewayOpenResponsesServerState>,
    heartbeat_enabled: bool,
    heartbeat_interval: Duration,
) -> MemoryDistillRuntimeHandle {
    if !heartbeat_enabled {
        state.record_memory_distill_disabled();
        return MemoryDistillRuntimeHandle::disabled();
    }

    let interval = heartbeat_interval.max(Duration::from_millis(500));
    let checkpoint_path = gateway_memory_distill_checkpoint_path(state.config.state_dir.as_path());
    let checkpoint_state = match load_memory_distill_checkpoint_state(checkpoint_path.as_path()) {
        Ok(checkpoint_state) => checkpoint_state,
        Err(error) => {
            state.record_memory_distill_checkpoint_load_failure();
            let _ = record_cortex_observer_event(
                state.config.state_dir.as_path(),
                "memory.distill.checkpoint_load_failed",
                json!({
                    "checkpoint_path": checkpoint_path.display().to_string(),
                    "error": error.to_string(),
                }),
            );
            MemoryDistillCheckpointState::default()
        }
    };

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let mut checkpoints = checkpoint_state;
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    state.record_memory_distill_cycle_start();
                    let cycle_report = run_memory_distill_cycle(
                        state.as_ref(),
                        checkpoint_path.as_path(),
                        &mut checkpoints,
                    );
                    state.record_memory_distill_cycle_report(&cycle_report);
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
    });

    MemoryDistillRuntimeHandle {
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
    }
}

impl GatewayOpenResponsesServerState {
    fn record_memory_distill_disabled(&self) {
        if let Ok(mut runtime) = self.memory_distill_runtime.lock() {
            runtime.enabled = false;
            runtime.in_flight = false;
        }
    }

    fn record_memory_distill_cycle_start(&self) {
        if let Ok(mut runtime) = self.memory_distill_runtime.lock() {
            runtime.enabled = true;
            runtime.in_flight = true;
            runtime.last_cycle_started_unix_ms = Some(current_unix_timestamp_ms());
        }
    }

    fn record_memory_distill_checkpoint_load_failure(&self) {
        if let Ok(mut runtime) = self.memory_distill_runtime.lock() {
            runtime.checkpoint_load_failures = runtime.checkpoint_load_failures.saturating_add(1);
            runtime.push_reason_code("memory_distill_checkpoint_load_failed");
        }
    }

    fn record_memory_distill_cycle_report(&self, report: &MemoryDistillCycleReport) {
        if let Ok(mut runtime) = self.memory_distill_runtime.lock() {
            runtime.enabled = true;
            runtime.in_flight = false;
            runtime.cycle_count = runtime.cycle_count.saturating_add(1);
            runtime.last_cycle_completed_unix_ms = Some(report.completed_unix_ms);
            runtime.last_cycle_sessions_scanned = report.sessions_scanned;
            runtime.last_cycle_entries_scanned = report.entries_scanned;
            runtime.last_cycle_candidates_extracted = report.candidates_extracted;
            runtime.last_cycle_writes_applied = report.writes_applied;
            runtime.last_cycle_write_failures = report.write_failures;
            runtime.sessions_scanned = runtime
                .sessions_scanned
                .saturating_add(report.sessions_scanned);
            runtime.entries_scanned = runtime
                .entries_scanned
                .saturating_add(report.entries_scanned);
            runtime.candidates_extracted = runtime
                .candidates_extracted
                .saturating_add(report.candidates_extracted);
            runtime.writes_applied = runtime.writes_applied.saturating_add(report.writes_applied);
            runtime.write_failures = runtime.write_failures.saturating_add(report.write_failures);
            runtime.session_load_failures = runtime
                .session_load_failures
                .saturating_add(report.session_load_failures);
            runtime.checkpoint_save_failures = runtime
                .checkpoint_save_failures
                .saturating_add(report.checkpoint_save_failures);
            runtime.last_sessions_with_updates = report.sessions_with_updates;
            for reason_code in &report.reason_codes {
                runtime.push_reason_code(reason_code.as_str());
            }
            for recent_write in &report.recent_writes {
                runtime.push_recent_write(recent_write.clone());
            }
        }
    }

    pub(super) fn collect_memory_distill_status_report(&self) -> GatewayMemoryDistillStatusReport {
        if let Ok(runtime) = self.memory_distill_runtime.lock() {
            return GatewayMemoryDistillStatusReport {
                enabled: runtime.enabled,
                in_flight: runtime.in_flight,
                cycle_count: runtime.cycle_count,
                last_cycle_started_unix_ms: runtime.last_cycle_started_unix_ms,
                last_cycle_completed_unix_ms: runtime.last_cycle_completed_unix_ms,
                sessions_scanned: runtime.sessions_scanned,
                entries_scanned: runtime.entries_scanned,
                candidates_extracted: runtime.candidates_extracted,
                writes_applied: runtime.writes_applied,
                write_failures: runtime.write_failures,
                last_cycle_sessions_scanned: runtime.last_cycle_sessions_scanned,
                last_cycle_entries_scanned: runtime.last_cycle_entries_scanned,
                last_cycle_candidates_extracted: runtime.last_cycle_candidates_extracted,
                last_cycle_writes_applied: runtime.last_cycle_writes_applied,
                last_cycle_write_failures: runtime.last_cycle_write_failures,
                session_load_failures: runtime.session_load_failures,
                checkpoint_load_failures: runtime.checkpoint_load_failures,
                checkpoint_save_failures: runtime.checkpoint_save_failures,
                last_sessions_with_updates: runtime.last_sessions_with_updates,
                last_reason_codes: runtime.last_reason_codes.clone(),
                recent_writes: runtime.recent_writes.clone(),
                checkpoint_path: gateway_memory_distill_checkpoint_path(
                    self.config.state_dir.as_path(),
                )
                .display()
                .to_string(),
            };
        }

        GatewayMemoryDistillStatusReport::default()
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct GatewayMemoryDistillRuntimeState {
    enabled: bool,
    in_flight: bool,
    cycle_count: u64,
    last_cycle_started_unix_ms: Option<u64>,
    last_cycle_completed_unix_ms: Option<u64>,
    sessions_scanned: u64,
    entries_scanned: u64,
    candidates_extracted: u64,
    writes_applied: u64,
    write_failures: u64,
    last_cycle_sessions_scanned: u64,
    last_cycle_entries_scanned: u64,
    last_cycle_candidates_extracted: u64,
    last_cycle_writes_applied: u64,
    last_cycle_write_failures: u64,
    session_load_failures: u64,
    checkpoint_load_failures: u64,
    checkpoint_save_failures: u64,
    last_sessions_with_updates: u64,
    last_reason_codes: Vec<String>,
    recent_writes: Vec<MemoryDistillRecentWrite>,
}

impl GatewayMemoryDistillRuntimeState {
    fn push_reason_code(&mut self, reason_code: &str) {
        let trimmed = reason_code.trim();
        if trimmed.is_empty() {
            return;
        }
        self.last_reason_codes.push(trimmed.to_string());
        truncate_front_to_limit(&mut self.last_reason_codes, MAX_REASON_CODES);
    }

    fn push_recent_write(&mut self, write: MemoryDistillRecentWrite) {
        self.recent_writes.push(write);
        truncate_front_to_limit(&mut self.recent_writes, MAX_RECENT_WRITES);
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub(super) struct GatewayMemoryDistillStatusReport {
    enabled: bool,
    in_flight: bool,
    cycle_count: u64,
    last_cycle_started_unix_ms: Option<u64>,
    last_cycle_completed_unix_ms: Option<u64>,
    sessions_scanned: u64,
    entries_scanned: u64,
    candidates_extracted: u64,
    writes_applied: u64,
    write_failures: u64,
    last_cycle_sessions_scanned: u64,
    last_cycle_entries_scanned: u64,
    last_cycle_candidates_extracted: u64,
    last_cycle_writes_applied: u64,
    last_cycle_write_failures: u64,
    session_load_failures: u64,
    checkpoint_load_failures: u64,
    checkpoint_save_failures: u64,
    last_sessions_with_updates: u64,
    last_reason_codes: Vec<String>,
    recent_writes: Vec<MemoryDistillRecentWrite>,
    checkpoint_path: String,
}

#[derive(Debug, Clone, Default)]
struct MemoryDistillCycleReport {
    completed_unix_ms: u64,
    sessions_scanned: u64,
    entries_scanned: u64,
    candidates_extracted: u64,
    writes_applied: u64,
    write_failures: u64,
    session_load_failures: u64,
    checkpoint_save_failures: u64,
    sessions_with_updates: u64,
    reason_codes: Vec<String>,
    recent_writes: Vec<MemoryDistillRecentWrite>,
}

impl MemoryDistillCycleReport {
    fn push_reason_code(&mut self, reason_code: &str) {
        let trimmed = reason_code.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.reason_codes.iter().any(|existing| existing == trimmed) {
            return;
        }
        self.reason_codes.push(trimmed.to_string());
    }
}

#[derive(Debug, Clone, Serialize, Default)]
struct MemoryDistillRecentWrite {
    session_key: String,
    entry_id: u64,
    memory_id: String,
    summary: String,
    memory_type: String,
    source_event_key: String,
    created: bool,
    observed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryDistillCheckpointFile {
    #[serde(default = "memory_distill_checkpoint_schema_version")]
    schema_version: u32,
    #[serde(default)]
    sessions: BTreeMap<String, MemoryDistillSessionCheckpoint>,
}

impl Default for MemoryDistillCheckpointFile {
    fn default() -> Self {
        Self {
            schema_version: memory_distill_checkpoint_schema_version(),
            sessions: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MemoryDistillSessionCheckpoint {
    last_entry_id: u64,
    updated_unix_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct MemoryDistillCheckpointState {
    sessions: BTreeMap<String, MemoryDistillSessionCheckpoint>,
}

#[derive(Debug, Clone)]
struct DistilledMemoryCandidate {
    kind: &'static str,
    summary: String,
    facts: Vec<String>,
    tags: Vec<String>,
    memory_type: MemoryType,
    importance: f32,
}

fn memory_distill_checkpoint_schema_version() -> u32 {
    MEMORY_DISTILL_CHECKPOINT_SCHEMA_VERSION
}

fn run_memory_distill_cycle(
    state: &GatewayOpenResponsesServerState,
    checkpoint_path: &Path,
    checkpoints: &mut MemoryDistillCheckpointState,
) -> MemoryDistillCycleReport {
    let mut report = MemoryDistillCycleReport::default();
    let state_dir = state.config.state_dir.as_path();
    let sessions_dir = state_dir.join("openresponses").join("sessions");

    let session_files = match list_gateway_session_files(sessions_dir.as_path()) {
        Ok(files) => files,
        Err(error) => {
            report.push_reason_code("memory_distill_session_directory_unreadable");
            let _ = record_cortex_observer_event(
                state_dir,
                "memory.distill.session_directory_unreadable",
                json!({
                    "sessions_dir": sessions_dir.display().to_string(),
                    "error": error.to_string(),
                }),
            );
            report.completed_unix_ms = current_unix_timestamp_ms();
            return report;
        }
    };

    let semantic_store = semantic_memory_store(state.config.memory_state_dir.as_path());
    for (session_key, session_path) in session_files {
        report.sessions_scanned = report.sessions_scanned.saturating_add(1);

        let mut checkpoint = checkpoints
            .sessions
            .get(session_key.as_str())
            .cloned()
            .unwrap_or_default();
        let previous_last_entry_id = checkpoint.last_entry_id;

        let store = match SessionStore::load(session_path.as_path()) {
            Ok(store) => store,
            Err(error) => {
                report.session_load_failures = report.session_load_failures.saturating_add(1);
                report.push_reason_code("memory_distill_session_load_failed");
                let _ = record_cortex_observer_event(
                    state_dir,
                    "memory.distill.session_load_failed",
                    json!({
                        "session_key": session_key,
                        "session_path": session_path.display().to_string(),
                        "error": error.to_string(),
                    }),
                );
                continue;
            }
        };

        for entry in store.entries() {
            if entry.id <= checkpoint.last_entry_id {
                continue;
            }
            report.entries_scanned = report.entries_scanned.saturating_add(1);
            let entry_processed = process_session_entry(
                state,
                session_key.as_str(),
                entry,
                semantic_store.clone(),
                &mut report,
            );
            if entry_processed {
                checkpoint.last_entry_id = entry.id;
                checkpoint.updated_unix_ms = current_unix_timestamp_ms();
            } else {
                break;
            }
        }

        if checkpoint.last_entry_id > previous_last_entry_id {
            report.sessions_with_updates = report.sessions_with_updates.saturating_add(1);
        }
        checkpoints.sessions.insert(session_key, checkpoint);
    }

    if let Err(error) = save_memory_distill_checkpoint_state(checkpoint_path, checkpoints) {
        report.checkpoint_save_failures = report.checkpoint_save_failures.saturating_add(1);
        report.push_reason_code("memory_distill_checkpoint_save_failed");
        let _ = record_cortex_observer_event(
            state_dir,
            "memory.distill.checkpoint_save_failed",
            json!({
                "checkpoint_path": checkpoint_path.display().to_string(),
                "error": error.to_string(),
            }),
        );
    }

    report.completed_unix_ms = current_unix_timestamp_ms();
    let _ = record_cortex_observer_event(
        state_dir,
        "memory.distill.cycle",
        json!({
            "sessions_scanned": report.sessions_scanned,
            "entries_scanned": report.entries_scanned,
            "candidates_extracted": report.candidates_extracted,
            "writes_applied": report.writes_applied,
            "write_failures": report.write_failures,
            "session_load_failures": report.session_load_failures,
            "checkpoint_save_failures": report.checkpoint_save_failures,
            "sessions_with_updates": report.sessions_with_updates,
            "reason_codes": report.reason_codes,
        }),
    );
    report
}

fn process_session_entry(
    state: &GatewayOpenResponsesServerState,
    session_key: &str,
    entry: &SessionEntry,
    semantic_store: tau_memory::runtime::FileMemoryStore,
    report: &mut MemoryDistillCycleReport,
) -> bool {
    if entry.message.role != MessageRole::User {
        return true;
    }

    let text = entry.message.text_content();
    let candidates = distill_candidates_from_user_text(text.as_str());
    if candidates.is_empty() {
        return true;
    }

    report.candidates_extracted = report
        .candidates_extracted
        .saturating_add(candidates.len() as u64);

    let scope = MemoryScope {
        workspace_id: session_key.to_string(),
        channel_id: "gateway".to_string(),
        actor_id: "user".to_string(),
    };
    let source_event_key = format!("session:{session_key}:entry:{}", entry.id);

    for (candidate_index, candidate) in candidates.iter().enumerate() {
        let memory_id =
            build_distilled_memory_id(session_key, entry.id, candidate, candidate_index);
        let memory_entry = MemoryEntry {
            memory_id: memory_id.clone(),
            summary: candidate.summary.clone(),
            tags: candidate.tags.clone(),
            facts: candidate.facts.clone(),
            source_event_key: source_event_key.clone(),
            recency_weight_bps: 0,
            confidence_bps: 950,
        };

        match semantic_store.write_entry_with_metadata(
            &scope,
            memory_entry,
            Some(candidate.memory_type),
            Some(candidate.importance),
        ) {
            Ok(write_result) => {
                report.writes_applied = report.writes_applied.saturating_add(1);
                report.recent_writes.push(MemoryDistillRecentWrite {
                    session_key: session_key.to_string(),
                    entry_id: entry.id,
                    memory_id: memory_id.clone(),
                    summary: candidate.summary.clone(),
                    memory_type: candidate.memory_type.as_str().to_string(),
                    source_event_key: source_event_key.clone(),
                    created: write_result.created,
                    observed_unix_ms: current_unix_timestamp_ms(),
                });
                truncate_front_to_limit(&mut report.recent_writes, MAX_RECENT_WRITES);
                record_cortex_memory_entry_write_event(
                    state.config.state_dir.as_path(),
                    session_key,
                    memory_id.as_str(),
                    write_result.created,
                );
                let _ = record_cortex_observer_event(
                    state.config.state_dir.as_path(),
                    "memory.distill.write",
                    json!({
                        "session_key": session_key,
                        "entry_id": entry.id,
                        "memory_id": memory_id,
                        "memory_type": candidate.memory_type.as_str(),
                        "created": write_result.created,
                        "source_event_key": source_event_key,
                    }),
                );
            }
            Err(error) => {
                report.write_failures = report.write_failures.saturating_add(1);
                report.push_reason_code("memory_distill_write_failed");
                let _ = record_cortex_observer_event(
                    state.config.state_dir.as_path(),
                    "memory.distill.write_failed",
                    json!({
                        "session_key": session_key,
                        "entry_id": entry.id,
                        "memory_id": memory_id,
                        "error": error.to_string(),
                    }),
                );
                return false;
            }
        }
    }

    true
}

fn list_gateway_session_files(
    sessions_dir: &Path,
) -> Result<Vec<(String, PathBuf)>, anyhow::Error> {
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    let entries = std::fs::read_dir(sessions_dir)
        .with_context(|| format!("failed to read {}", sessions_dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_jsonl = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("jsonl"))
            .unwrap_or(false);
        if !is_jsonl {
            continue;
        }
        let Some(session_key) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let trimmed_session_key = session_key.trim();
        if trimmed_session_key.is_empty() {
            continue;
        }
        sessions.push((trimmed_session_key.to_string(), path));
    }

    sessions.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(sessions)
}

fn gateway_memory_distill_checkpoint_path(state_dir: &Path) -> PathBuf {
    state_dir
        .join("openresponses")
        .join("memory-distill-checkpoints.json")
}

fn load_memory_distill_checkpoint_state(
    path: &Path,
) -> Result<MemoryDistillCheckpointState, anyhow::Error> {
    if !path.exists() {
        return Ok(MemoryDistillCheckpointState::default());
    }

    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(MemoryDistillCheckpointState::default());
    }

    let parsed: MemoryDistillCheckpointFile = serde_json::from_str(raw.as_str())
        .with_context(|| format!("failed to parse {}", path.display()))?;
    if parsed.schema_version != MEMORY_DISTILL_CHECKPOINT_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported memory distill checkpoint schema_version {} in {} (expected {})",
            parsed.schema_version,
            path.display(),
            MEMORY_DISTILL_CHECKPOINT_SCHEMA_VERSION
        );
    }

    Ok(MemoryDistillCheckpointState {
        sessions: parsed.sessions,
    })
}

fn save_memory_distill_checkpoint_state(
    path: &Path,
    state: &MemoryDistillCheckpointState,
) -> Result<(), anyhow::Error> {
    let payload = MemoryDistillCheckpointFile {
        schema_version: MEMORY_DISTILL_CHECKPOINT_SCHEMA_VERSION,
        sessions: state.sessions.clone(),
    };
    let encoded = serde_json::to_string_pretty(&payload)
        .context("failed to serialize memory distill checkpoint")?;
    write_text_atomic(path, encoded.as_str())
        .with_context(|| format!("failed to write {}", path.display()))
}

fn distill_candidates_from_user_text(text: &str) -> Vec<DistilledMemoryCandidate> {
    let normalized = normalize_whitespace(text);
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();

    if let Some(value) = extract_phrase_value(&normalized, &["my birthday is", "my bday is"]) {
        candidates.push(DistilledMemoryCandidate {
            kind: "birthday",
            summary: format!("User birthday is {value}"),
            facts: vec![format!("birthday={value}")],
            tags: vec!["profile".to_string(), "birthday".to_string()],
            memory_type: MemoryType::Identity,
            importance: 0.95,
        });
    }

    if let Some(value) = extract_phrase_value(&normalized, &["my name is"]) {
        candidates.push(DistilledMemoryCandidate {
            kind: "name",
            summary: format!("User name is {value}"),
            facts: vec![format!("name={value}")],
            tags: vec!["profile".to_string(), "identity".to_string()],
            memory_type: MemoryType::Identity,
            importance: 0.9,
        });
    }

    if let Some(value) = extract_phrase_value(&normalized, &["call me"]) {
        candidates.push(DistilledMemoryCandidate {
            kind: "name",
            summary: format!("User name is {value}"),
            facts: vec![format!("name={value}")],
            tags: vec![
                "profile".to_string(),
                "identity".to_string(),
                "alias".to_string(),
            ],
            memory_type: MemoryType::Identity,
            importance: 0.9,
        });
    }

    if let Some(value) = extract_phrase_value(&normalized, &["i prefer"]) {
        candidates.push(DistilledMemoryCandidate {
            kind: "preference",
            summary: format!("User prefers {value}"),
            facts: vec![value.clone()],
            tags: vec!["preference".to_string()],
            memory_type: MemoryType::Preference,
            importance: 0.8,
        });
    }

    if let Some(value) = extract_phrase_value(&normalized, &["my goal is", "i want to"]) {
        if !value.ends_with('?') {
            candidates.push(DistilledMemoryCandidate {
                kind: "goal",
                summary: format!("User goal: {value}"),
                facts: vec![value.clone()],
                tags: vec!["goal".to_string()],
                memory_type: MemoryType::Goal,
                importance: 0.85,
            });
        }
    }

    if let Some(value) = extract_phrase_value(&normalized, &["i can't", "i cannot"]) {
        candidates.push(DistilledMemoryCandidate {
            kind: "constraint",
            summary: format!("User constraint: {value}"),
            facts: vec![value.clone()],
            tags: vec!["constraint".to_string()],
            memory_type: MemoryType::Fact,
            importance: 0.7,
        });
    }

    if let Some(value) =
        extract_phrase_value(&normalized, &["i am based in", "i'm based in", "i live in"])
    {
        candidates.push(DistilledMemoryCandidate {
            kind: "location",
            summary: format!("User is based in {value}"),
            facts: vec![format!("location={value}")],
            tags: vec!["profile".to_string(), "location".to_string()],
            memory_type: MemoryType::Fact,
            importance: 0.75,
        });
    }

    if let Some(value) = extract_phrase_value(&normalized, &["i am allergic to", "i'm allergic to"])
    {
        candidates.push(DistilledMemoryCandidate {
            kind: "constraint",
            summary: format!("User is allergic to {value}"),
            facts: vec![format!("allergy={value}")],
            tags: vec!["constraint".to_string(), "allergy".to_string()],
            memory_type: MemoryType::Fact,
            importance: 0.8,
        });
    }

    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for candidate in candidates {
        let key = candidate.summary.to_ascii_lowercase();
        if seen.insert(key) {
            deduped.push(candidate);
        }
    }
    deduped
}

fn extract_phrase_value(text: &str, phrases: &[&str]) -> Option<String> {
    let lowercase = text.to_ascii_lowercase();
    for phrase in phrases {
        let phrase_lower = phrase.to_ascii_lowercase();
        if let Some(index) = lowercase.find(phrase_lower.as_str()) {
            let start = index.saturating_add(phrase_lower.len());
            let tail = text.get(start..).unwrap_or_default();
            if let Some(value) = sanitize_distilled_value(tail) {
                return Some(value);
            }
        }
    }
    None
}

fn sanitize_distilled_value(raw_tail: &str) -> Option<String> {
    let mut end_index = raw_tail.len();
    for marker in ['\n', '.', '!', '?'] {
        if let Some(index) = raw_tail.find(marker) {
            end_index = end_index.min(index);
        }
    }
    let mut value = raw_tail
        .get(..end_index)
        .unwrap_or_default()
        .trim()
        .to_string();
    value = value
        .trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == '`' || ch == ',' || ch == ':')
        .trim()
        .to_string();
    if value.len() < 2 {
        return None;
    }
    if value.len() > 120 {
        value.truncate(120);
        value = value.trim().to_string();
    }
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn normalize_whitespace(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_front_to_limit<T>(items: &mut Vec<T>, limit: usize) {
    if items.len() <= limit {
        return;
    }
    let drop_count = items.len().saturating_sub(limit);
    items.drain(0..drop_count);
}

fn build_distilled_memory_id(
    session_key: &str,
    entry_id: u64,
    candidate: &DistilledMemoryCandidate,
    candidate_index: usize,
) -> String {
    let material = format!(
        "session={session_key}|entry_id={entry_id}|kind={}|index={candidate_index}|summary={}",
        candidate.kind, candidate.summary
    );
    let digest = fnv1a64_hex(material.as_bytes());
    format!(
        "auto:{}:entry:{}:{}:{}",
        session_key, entry_id, candidate.kind, digest
    )
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{
        distill_candidates_from_user_text, gateway_memory_distill_checkpoint_path,
        load_memory_distill_checkpoint_state, run_memory_distill_cycle,
        MemoryDistillCheckpointState,
    };
    use std::sync::Arc;
    use tau_ai::{ChatRequest, ChatResponse, ChatUsage, LlmClient, Message, TauAiError};

    use crate::gateway_openresponses::{
        GatewayOpenResponsesAuthMode, GatewayOpenResponsesServerConfig,
        GatewayOpenResponsesServerState, NoopGatewayToolRegistrar,
    };

    #[derive(Clone)]
    struct NoopTestClient;

    #[async_trait::async_trait]
    impl LlmClient for NoopTestClient {
        async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse, TauAiError> {
            Ok(ChatResponse {
                message: Message::assistant_text("noop"),
                finish_reason: Some("stop".to_string()),
                usage: ChatUsage::default(),
            })
        }
    }

    #[test]
    fn unit_distill_candidates_extracts_identity_preference_goal_and_constraint() {
        let candidates = distill_candidates_from_user_text(
            "Remember this: my birthday is April 3. I prefer concise answers. My goal is ship this release. I can't use sudo.",
        );

        let summaries = candidates
            .iter()
            .map(|candidate| candidate.summary.as_str())
            .collect::<Vec<_>>();
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("birthday is April 3")));
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("prefers concise answers")));
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("goal: ship this release")));
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("constraint: use sudo")));
    }

    #[test]
    fn unit_distill_candidates_extracts_alias_location_and_allergy_constraint() {
        let candidates = distill_candidates_from_user_text(
            "Call me Niko. I am based in Austin, Texas. I am allergic to peanuts.",
        );

        let summaries = candidates
            .iter()
            .map(|candidate| candidate.summary.as_str())
            .collect::<Vec<_>>();
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("name is Niko")));
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("based in Austin, Texas")));
        assert!(summaries
            .iter()
            .any(|summary| summary.contains("allergic to peanuts")));
    }

    #[tokio::test]
    async fn integration_memory_distill_cycle_writes_memory_and_checkpoints_processed_entries() {
        let temp = tempfile::tempdir().expect("tempdir");
        let state_dir = temp.path().join(".tau/gateway");
        let memory_state_dir = temp.path().join(".tau/memory");
        let session_path = state_dir
            .join("openresponses")
            .join("sessions")
            .join("alpha.jsonl");
        if let Some(parent) = session_path.parent() {
            std::fs::create_dir_all(parent).expect("create session dir");
        }

        let mut store = tau_session::SessionStore::load(&session_path).expect("load session");
        let mut head = store
            .append_messages(None, &[Message::system("system prompt")])
            .expect("append system");
        head = store
            .append_messages(
                head,
                &[Message::user(
                    "Please remember my birthday is April 3 and I prefer concise answers.",
                )],
            )
            .expect("append user");
        let _ = store
            .append_messages(head, &[Message::assistant_text("ack")])
            .expect("append assistant");

        let state = GatewayOpenResponsesServerState::new(GatewayOpenResponsesServerConfig {
            client: Arc::new(NoopTestClient),
            model: "openai/gpt-5.2".to_string(),
            model_input_cost_per_million: None,
            model_cached_input_cost_per_million: None,
            model_output_cost_per_million: None,
            system_prompt: "You are Tau".to_string(),
            max_turns: 4,
            tool_registrar: Arc::new(NoopGatewayToolRegistrar),
            turn_timeout_ms: 0,
            session_lock_wait_ms: 500,
            session_lock_stale_ms: 10_000,
            state_dir: state_dir.clone(),
            memory_state_dir: memory_state_dir.clone(),
            bind: "127.0.0.1:0".to_string(),
            auth_mode: GatewayOpenResponsesAuthMode::LocalhostDev,
            auth_token: None,
            auth_password: None,
            session_ttl_seconds: 3_600,
            rate_limit_window_seconds: 60,
            rate_limit_max_requests: 120,
            max_input_chars: 12_000,
            runtime_heartbeat: tau_runtime::RuntimeHeartbeatSchedulerConfig {
                enabled: true,
                interval: std::time::Duration::from_secs(1),
                state_path: temp.path().join(".tau/runtime-heartbeat/state.json"),
                ..tau_runtime::RuntimeHeartbeatSchedulerConfig::default()
            },
            external_coding_agent_bridge: tau_runtime::ExternalCodingAgentBridgeConfig::default(),
        });

        let checkpoint_path = gateway_memory_distill_checkpoint_path(state_dir.as_path());
        let mut checkpoints = MemoryDistillCheckpointState::default();
        let first_report =
            run_memory_distill_cycle(&state, checkpoint_path.as_path(), &mut checkpoints);

        assert_eq!(first_report.sessions_scanned, 1);
        assert!(
            first_report.writes_applied >= 2,
            "expected two distilled writes"
        );
        assert_eq!(first_report.write_failures, 0);

        let semantic_store =
            crate::gateway_openresponses::semantic_memory_store(memory_state_dir.as_path());
        let records = semantic_store
            .list_latest_records(None, usize::MAX)
            .expect("list memory records");
        assert!(records
            .iter()
            .any(|record| record.entry.summary.contains("birthday is April 3")));
        assert!(records
            .iter()
            .any(|record| record.entry.summary.contains("prefers concise answers")));

        state.record_memory_distill_cycle_report(&first_report);
        let status_report = state.collect_memory_distill_status_report();
        assert!(status_report.last_cycle_writes_applied >= 2);
        assert!(status_report.recent_writes.len() >= 2);
        assert!(status_report
            .recent_writes
            .iter()
            .any(|write| write.summary.contains("birthday is April 3")));

        let second_report =
            run_memory_distill_cycle(&state, checkpoint_path.as_path(), &mut checkpoints);
        assert_eq!(second_report.writes_applied, 0);
        assert_eq!(second_report.write_failures, 0);

        let persisted = load_memory_distill_checkpoint_state(checkpoint_path.as_path())
            .expect("load checkpoint state");
        let alpha_checkpoint = persisted
            .sessions
            .get("alpha")
            .expect("checkpoint for alpha session");
        assert!(alpha_checkpoint.last_entry_id >= 3);
    }
}
