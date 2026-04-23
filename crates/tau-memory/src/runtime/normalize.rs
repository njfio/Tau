//! Normalization helpers used by the file-backed memory store for scope,
//! entry, and relation canonicalization, plus the shared
//! `current_unix_timestamp_ms()` clock helper.
//!
//! Extracted from `runtime.rs` as part of the incremental god-file split
//! documented in `docs/planning/god-file-split-audit-2026-04-23.md`.
//! No behavioral change.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};

use crate::memory_contract::{MemoryEntry, MemoryScope};

use super::{
    MemoryRelation, MemoryRelationInput, MemoryRelationType, MEMORY_INVALID_RELATION_REASON_CODE,
    MEMORY_SCOPE_DEFAULT_ACTOR, MEMORY_SCOPE_DEFAULT_CHANNEL, MEMORY_SCOPE_DEFAULT_WORKSPACE,
};

pub(super) fn normalize_scope(scope: &MemoryScope) -> MemoryScope {
    MemoryScope {
        workspace_id: normalize_scope_component(
            &scope.workspace_id,
            MEMORY_SCOPE_DEFAULT_WORKSPACE,
        ),
        channel_id: normalize_scope_component(&scope.channel_id, MEMORY_SCOPE_DEFAULT_CHANNEL),
        actor_id: normalize_scope_component(&scope.actor_id, MEMORY_SCOPE_DEFAULT_ACTOR),
    }
}

pub(super) fn sqlite_i64_from_u64(value: u64, field: &str) -> Result<i64> {
    i64::try_from(value)
        .with_context(|| format!("{field} value {value} exceeds SQLite INTEGER range"))
}

pub(super) fn sqlite_u64_from_i64(value: i64, field: &str) -> Result<u64> {
    u64::try_from(value)
        .with_context(|| format!("{field} value {value} must be non-negative SQLite INTEGER"))
}

fn normalize_scope_component(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

pub(super) fn normalize_entry(entry: MemoryEntry) -> Result<MemoryEntry> {
    let memory_id = entry.memory_id.trim().to_string();
    if memory_id.is_empty() {
        bail!("memory_id must not be empty");
    }
    let summary = entry.summary.trim().to_string();
    if summary.is_empty() {
        bail!("summary must not be empty");
    }

    let tags = entry
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    let facts = entry
        .facts
        .into_iter()
        .map(|fact| fact.trim().to_string())
        .filter(|fact| !fact.is_empty())
        .collect::<Vec<_>>();

    Ok(MemoryEntry {
        memory_id,
        summary,
        tags,
        facts,
        source_event_key: entry.source_event_key.trim().to_string(),
        recency_weight_bps: entry.recency_weight_bps,
        confidence_bps: entry.confidence_bps,
    })
}

pub(super) fn normalize_relations(
    source_memory_id: &str,
    relations: &[MemoryRelationInput],
    known_memory_ids: &BTreeSet<String>,
) -> Result<Vec<MemoryRelation>> {
    if relations.is_empty() {
        return Ok(Vec::new());
    }

    let mut deduped = BTreeMap::<(String, MemoryRelationType), MemoryRelation>::new();
    for relation in relations {
        let target_id = relation.target_id.trim();
        if target_id.is_empty() {
            bail!("{MEMORY_INVALID_RELATION_REASON_CODE}: relation target_id must not be empty");
        }
        if target_id == source_memory_id {
            bail!("{MEMORY_INVALID_RELATION_REASON_CODE}: source_id and target_id must differ");
        }
        if !known_memory_ids.contains(target_id) {
            bail!(
                "{MEMORY_INVALID_RELATION_REASON_CODE}: unknown target_id '{}'",
                target_id
            );
        }

        let relation_type = normalize_relation_type(relation.relation_type.as_deref())?;
        let raw_weight = relation.weight.unwrap_or(1.0);
        if !raw_weight.is_finite() {
            bail!("{MEMORY_INVALID_RELATION_REASON_CODE}: relation weight must be finite");
        }
        if !(0.0..=1.0).contains(&raw_weight) {
            bail!(
                "{MEMORY_INVALID_RELATION_REASON_CODE}: relation weight must be in range 0.0..=1.0"
            );
        }
        let effective_weight = raw_weight.clamp(0.0, 1.0);
        deduped.insert(
            (target_id.to_string(), relation_type),
            MemoryRelation {
                target_id: target_id.to_string(),
                relation_type,
                weight: raw_weight,
                effective_weight,
            },
        );
    }

    Ok(deduped.into_values().collect())
}

fn normalize_relation_type(value: Option<&str>) -> Result<MemoryRelationType> {
    let normalized = value.unwrap_or("related_to").trim().to_ascii_lowercase();
    if normalized.is_empty() {
        bail!("{MEMORY_INVALID_RELATION_REASON_CODE}: relation_type must not be empty");
    }
    MemoryRelationType::parse(normalized.as_str()).ok_or_else(|| {
        anyhow!(
            "{MEMORY_INVALID_RELATION_REASON_CODE}: unsupported relation_type '{}'",
            normalized
        )
    })
}

pub(super) fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
