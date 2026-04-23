//! Serde default functions used by the runtime's DTO structs.
//!
//! Extracted from `runtime.rs` as part of the incremental god-file split
//! documented in `docs/planning/god-file-split-audit-2026-04-23.md`.
//! No behavioral change — these functions are identical to the ones they replace.

use crate::memory_contract::MemoryScope;

use super::{
    MemoryType, MEMORY_EMBEDDING_REASON_HASH_ONLY, MEMORY_EMBEDDING_SOURCE_HASH,
    MEMORY_GRAPH_SIGNAL_WEIGHT_DEFAULT, MEMORY_INGESTION_DEFAULT_CHUNK_LINE_COUNT,
    MEMORY_INGESTION_LLM_DEFAULT_TIMEOUT_MS, MEMORY_LIFECYCLE_DEFAULT_DECAY_RATE,
    MEMORY_LIFECYCLE_DEFAULT_DUPLICATE_SIMILARITY_THRESHOLD,
    MEMORY_LIFECYCLE_DEFAULT_ORPHAN_IMPORTANCE_FLOOR,
    MEMORY_LIFECYCLE_DEFAULT_PRUNE_IMPORTANCE_FLOOR, MEMORY_LIFECYCLE_DEFAULT_STALE_AFTER_MS,
    MEMORY_SCOPE_DEFAULT_ACTOR, MEMORY_SCOPE_DEFAULT_CHANNEL, MEMORY_SCOPE_DEFAULT_WORKSPACE,
};

pub(super) fn default_embedding_source() -> String {
    MEMORY_EMBEDDING_SOURCE_HASH.to_string()
}

pub(super) fn default_embedding_reason_code() -> String {
    MEMORY_EMBEDDING_REASON_HASH_ONLY.to_string()
}

pub(super) fn default_memory_importance() -> f32 {
    MemoryType::default().default_importance()
}

pub(super) fn default_graph_signal_weight() -> f32 {
    MEMORY_GRAPH_SIGNAL_WEIGHT_DEFAULT
}

pub(super) fn default_lifecycle_stale_after_unix_ms() -> u64 {
    MEMORY_LIFECYCLE_DEFAULT_STALE_AFTER_MS
}

pub(super) fn default_lifecycle_decay_rate() -> f32 {
    MEMORY_LIFECYCLE_DEFAULT_DECAY_RATE
}

pub(super) fn default_lifecycle_prune_importance_floor() -> f32 {
    MEMORY_LIFECYCLE_DEFAULT_PRUNE_IMPORTANCE_FLOOR
}

pub(super) fn default_lifecycle_orphan_importance_floor() -> f32 {
    MEMORY_LIFECYCLE_DEFAULT_ORPHAN_IMPORTANCE_FLOOR
}

pub(super) fn default_lifecycle_orphan_cleanup_enabled() -> bool {
    true
}

pub(super) fn default_lifecycle_duplicate_cleanup_enabled() -> bool {
    false
}

pub(super) fn default_lifecycle_duplicate_similarity_threshold() -> f32 {
    MEMORY_LIFECYCLE_DEFAULT_DUPLICATE_SIMILARITY_THRESHOLD
}

pub(super) fn default_ingestion_scope() -> MemoryScope {
    MemoryScope {
        workspace_id: MEMORY_SCOPE_DEFAULT_WORKSPACE.to_string(),
        channel_id: MEMORY_SCOPE_DEFAULT_CHANNEL.to_string(),
        actor_id: MEMORY_SCOPE_DEFAULT_ACTOR.to_string(),
    }
}

pub(super) fn default_ingestion_chunk_line_count() -> usize {
    MEMORY_INGESTION_DEFAULT_CHUNK_LINE_COUNT
}

pub(super) fn default_ingestion_delete_source_on_success() -> bool {
    true
}

pub(super) fn default_ingestion_llm_timeout_ms() -> u64 {
    MEMORY_INGESTION_LLM_DEFAULT_TIMEOUT_MS
}
