//! Runtime data types for the tau-memory runtime.
//!
//! Extracted from `runtime.rs` per the 2026-04-23 split audit (Proposal 3,
//! increment 3). Pure declarations + their inherent `impl` blocks + `Default`
//! impls. No business logic lives here; it stays in `runtime.rs` and its
//! sibling modules.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::defaults::{
    default_embedding_reason_code, default_embedding_source, default_graph_signal_weight,
    default_ingestion_chunk_line_count, default_ingestion_delete_source_on_success,
    default_ingestion_llm_timeout_ms, default_ingestion_scope, default_lifecycle_decay_rate,
    default_lifecycle_duplicate_cleanup_enabled, default_lifecycle_duplicate_similarity_threshold,
    default_lifecycle_orphan_cleanup_enabled, default_lifecycle_orphan_importance_floor,
    default_lifecycle_prune_importance_floor, default_lifecycle_stale_after_unix_ms,
    default_memory_importance,
};
use super::MEMORY_GRAPH_SIGNAL_WEIGHT_DEFAULT;
use crate::memory_contract::{MemoryEntry, MemoryScope};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Enumerates supported `MemoryStorageBackend` values.
pub enum MemoryStorageBackend {
    Jsonl,
    Sqlite,
}

impl MemoryStorageBackend {
    pub fn label(self) -> &'static str {
        match self {
            MemoryStorageBackend::Jsonl => "jsonl",
            MemoryStorageBackend::Sqlite => "sqlite",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedMemoryBackend {
    pub(super) backend: MemoryStorageBackend,
    pub(super) storage_path: Option<PathBuf>,
    pub(super) reason_code: String,
}

/// Enumerates supported canonical `MemoryRelationType` values.
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRelationType {
    #[default]
    RelatedTo,
    Updates,
    Contradicts,
    CausedBy,
    ResultOf,
    PartOf,
}

impl MemoryRelationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RelatedTo => "related_to",
            Self::Updates => "updates",
            Self::Contradicts => "contradicts",
            Self::CausedBy => "caused_by",
            Self::ResultOf => "result_of",
            Self::PartOf => "part_of",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "related_to" | "relates_to" | "supports" | "references" => Some(Self::RelatedTo),
            "updates" => Some(Self::Updates),
            "contradicts" | "blocks" => Some(Self::Contradicts),
            "caused_by" | "depends_on" => Some(Self::CausedBy),
            "result_of" => Some(Self::ResultOf),
            "part_of" => Some(Self::PartOf),
            _ => None,
        }
    }
}

/// Public struct `MemoryRelation` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRelation {
    pub target_id: String,
    pub relation_type: MemoryRelationType,
    pub weight: f32,
    pub effective_weight: f32,
}

/// Public struct `MemoryRelationInput` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRelationInput {
    pub target_id: String,
    #[serde(default)]
    pub relation_type: Option<String>,
    #[serde(default)]
    pub weight: Option<f32>,
}

/// Enumerates supported `MemoryType` values.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Identity,
    Goal,
    Decision,
    Todo,
    Preference,
    Fact,
    Event,
    #[default]
    Observation,
}

impl MemoryType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Goal => "goal",
            Self::Decision => "decision",
            Self::Todo => "todo",
            Self::Preference => "preference",
            Self::Fact => "fact",
            Self::Event => "event",
            Self::Observation => "observation",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "identity" => Some(Self::Identity),
            "goal" => Some(Self::Goal),
            "decision" => Some(Self::Decision),
            "todo" => Some(Self::Todo),
            "preference" => Some(Self::Preference),
            "fact" => Some(Self::Fact),
            "event" => Some(Self::Event),
            "observation" => Some(Self::Observation),
            _ => None,
        }
    }

    pub fn default_importance(self) -> f32 {
        match self {
            Self::Identity => 1.0,
            Self::Goal => 0.9,
            Self::Decision => 0.85,
            Self::Todo => 0.8,
            Self::Preference => 0.7,
            Self::Fact => 0.65,
            Self::Event => 0.55,
            Self::Observation => 0.3,
        }
    }
}

/// Public struct `MemoryTypeImportanceProfile` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryTypeImportanceProfile {
    pub identity: f32,
    pub goal: f32,
    pub decision: f32,
    pub todo: f32,
    pub preference: f32,
    pub fact: f32,
    pub event: f32,
    pub observation: f32,
}

impl Default for MemoryTypeImportanceProfile {
    fn default() -> Self {
        Self {
            identity: MemoryType::Identity.default_importance(),
            goal: MemoryType::Goal.default_importance(),
            decision: MemoryType::Decision.default_importance(),
            todo: MemoryType::Todo.default_importance(),
            preference: MemoryType::Preference.default_importance(),
            fact: MemoryType::Fact.default_importance(),
            event: MemoryType::Event.default_importance(),
            observation: MemoryType::Observation.default_importance(),
        }
    }
}

impl MemoryTypeImportanceProfile {
    pub fn importance_for(&self, memory_type: MemoryType) -> f32 {
        match memory_type {
            MemoryType::Identity => self.identity,
            MemoryType::Goal => self.goal,
            MemoryType::Decision => self.decision,
            MemoryType::Todo => self.todo,
            MemoryType::Preference => self.preference,
            MemoryType::Fact => self.fact,
            MemoryType::Event => self.event,
            MemoryType::Observation => self.observation,
        }
    }

    pub fn set_importance(&mut self, memory_type: MemoryType, value: f32) {
        match memory_type {
            MemoryType::Identity => self.identity = value,
            MemoryType::Goal => self.goal = value,
            MemoryType::Decision => self.decision = value,
            MemoryType::Todo => self.todo = value,
            MemoryType::Preference => self.preference = value,
            MemoryType::Fact => self.fact = value,
            MemoryType::Event => self.event = value,
            MemoryType::Observation => self.observation = value,
        }
    }

    pub fn validate(&self) -> Result<()> {
        for (label, value) in [
            ("identity", self.identity),
            ("goal", self.goal),
            ("decision", self.decision),
            ("todo", self.todo),
            ("preference", self.preference),
            ("fact", self.fact),
            ("event", self.event),
            ("observation", self.observation),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                bail!(
                    "memory type default importance for '{}' must be finite and within 0.0..=1.0 (received {})",
                    label,
                    value
                );
            }
        }
        Ok(())
    }
}

pub(super) fn importance_rank_multiplier(importance: f32) -> f32 {
    1.0 + importance.clamp(0.0, 1.0)
}

/// Public struct `MemoryEmbeddingProviderConfig` used across Tau components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEmbeddingProviderConfig {
    pub provider: String,
    pub model: String,
    pub api_base: String,
    pub api_key: String,
    pub dimensions: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ComputedEmbedding {
    pub(super) vector: Vec<f32>,
    pub(super) backend: String,
    pub(super) model: Option<String>,
    pub(super) reason_code: String,
}

/// Public struct `RuntimeMemoryRecord` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeMemoryRecord {
    pub schema_version: u32,
    pub updated_unix_ms: u64,
    pub scope: MemoryScope,
    pub entry: MemoryEntry,
    #[serde(default)]
    pub memory_type: MemoryType,
    #[serde(default = "default_memory_importance")]
    pub importance: f32,
    #[serde(default = "default_embedding_source")]
    pub embedding_source: String,
    #[serde(default)]
    pub embedding_model: Option<String>,
    #[serde(default)]
    pub embedding_vector: Vec<f32>,
    #[serde(default = "default_embedding_reason_code")]
    pub embedding_reason_code: String,
    #[serde(default)]
    pub last_accessed_at_unix_ms: u64,
    #[serde(default)]
    pub access_count: u64,
    #[serde(default)]
    pub forgotten: bool,
    #[serde(default)]
    pub relations: Vec<MemoryRelation>,
}

/// Public struct `MemoryScopeFilter` used across Tau components.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryScopeFilter {
    pub workspace_id: Option<String>,
    pub channel_id: Option<String>,
    pub actor_id: Option<String>,
}

impl MemoryScopeFilter {
    /// Returns true when `scope` satisfies the configured filter dimensions.
    pub fn matches_scope(&self, scope: &MemoryScope) -> bool {
        let matches_workspace = self
            .workspace_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value == scope.workspace_id)
            .unwrap_or(true);
        if !matches_workspace {
            return false;
        }

        let matches_channel = self
            .channel_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value == scope.channel_id)
            .unwrap_or(true);
        if !matches_channel {
            return false;
        }

        self.actor_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value == scope.actor_id)
            .unwrap_or(true)
    }
}

/// Public struct `MemoryWriteResult` used across Tau components.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryWriteResult {
    pub record: RuntimeMemoryRecord,
    pub created: bool,
}

/// Public struct `MemorySearchOptions` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemorySearchOptions {
    pub scope: MemoryScopeFilter,
    pub limit: usize,
    pub embedding_dimensions: usize,
    pub min_similarity: f32,
    pub enable_hybrid_retrieval: bool,
    pub bm25_k1: f32,
    pub bm25_b: f32,
    pub bm25_min_score: f32,
    pub rrf_k: usize,
    pub rrf_vector_weight: f32,
    pub rrf_lexical_weight: f32,
    #[serde(default = "default_graph_signal_weight")]
    pub graph_signal_weight: f32,
    pub enable_embedding_migration: bool,
    pub benchmark_against_hash: bool,
    pub benchmark_against_vector_only: bool,
}

impl Default for MemorySearchOptions {
    fn default() -> Self {
        Self {
            scope: MemoryScopeFilter::default(),
            limit: 5,
            embedding_dimensions: 128,
            min_similarity: 0.55,
            enable_hybrid_retrieval: false,
            bm25_k1: 1.2,
            bm25_b: 0.75,
            bm25_min_score: 0.0,
            rrf_k: 60,
            rrf_vector_weight: 1.0,
            rrf_lexical_weight: 1.0,
            graph_signal_weight: MEMORY_GRAPH_SIGNAL_WEIGHT_DEFAULT,
            enable_embedding_migration: true,
            benchmark_against_hash: false,
            benchmark_against_vector_only: false,
        }
    }
}

/// Public struct `MemoryLifecycleMaintenancePolicy` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryLifecycleMaintenancePolicy {
    #[serde(default = "default_lifecycle_stale_after_unix_ms")]
    pub stale_after_unix_ms: u64,
    #[serde(default = "default_lifecycle_decay_rate")]
    pub decay_rate: f32,
    #[serde(default = "default_lifecycle_prune_importance_floor")]
    pub prune_importance_floor: f32,
    #[serde(default = "default_lifecycle_orphan_cleanup_enabled")]
    pub orphan_cleanup_enabled: bool,
    #[serde(default = "default_lifecycle_orphan_importance_floor")]
    pub orphan_importance_floor: f32,
    #[serde(default = "default_lifecycle_duplicate_cleanup_enabled")]
    pub duplicate_cleanup_enabled: bool,
    #[serde(default = "default_lifecycle_duplicate_similarity_threshold")]
    pub duplicate_similarity_threshold: f32,
}

impl Default for MemoryLifecycleMaintenancePolicy {
    fn default() -> Self {
        Self {
            stale_after_unix_ms: default_lifecycle_stale_after_unix_ms(),
            decay_rate: default_lifecycle_decay_rate(),
            prune_importance_floor: default_lifecycle_prune_importance_floor(),
            orphan_cleanup_enabled: default_lifecycle_orphan_cleanup_enabled(),
            orphan_importance_floor: default_lifecycle_orphan_importance_floor(),
            duplicate_cleanup_enabled: default_lifecycle_duplicate_cleanup_enabled(),
            duplicate_similarity_threshold: default_lifecycle_duplicate_similarity_threshold(),
        }
    }
}

/// Public struct `MemoryLifecycleMaintenanceResult` used across Tau components.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryLifecycleMaintenanceResult {
    pub scanned_records: usize,
    pub decayed_records: usize,
    pub pruned_records: usize,
    pub orphan_forgotten_records: usize,
    pub duplicate_forgotten_records: usize,
    pub identity_exempt_records: usize,
    pub updated_records: usize,
    pub unchanged_records: usize,
}

/// Public struct `MemoryIngestionOptions` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryIngestionOptions {
    #[serde(default = "default_ingestion_scope")]
    pub scope: MemoryScope,
    #[serde(default = "default_ingestion_chunk_line_count")]
    pub chunk_line_count: usize,
    #[serde(default = "default_ingestion_delete_source_on_success")]
    pub delete_source_on_success: bool,
}

impl Default for MemoryIngestionOptions {
    fn default() -> Self {
        Self {
            scope: default_ingestion_scope(),
            chunk_line_count: default_ingestion_chunk_line_count(),
            delete_source_on_success: default_ingestion_delete_source_on_success(),
        }
    }
}

/// Public struct `MemoryIngestionLlmOptions` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryIngestionLlmOptions {
    pub provider: String,
    pub model: String,
    pub api_base: String,
    pub api_key: String,
    #[serde(default = "default_ingestion_llm_timeout_ms")]
    pub timeout_ms: u64,
}

/// Public struct `MemoryIngestionWatchPollingState` used across Tau components.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryIngestionWatchPollingState {
    #[serde(default)]
    pub file_fingerprints: BTreeMap<String, String>,
}

/// Public struct `MemoryIngestionResult` used across Tau components.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryIngestionResult {
    pub discovered_files: usize,
    pub supported_files: usize,
    pub skipped_unsupported_files: usize,
    pub processed_files: usize,
    pub deleted_files: usize,
    pub chunks_discovered: usize,
    pub chunks_ingested: usize,
    pub chunks_skipped_existing: usize,
    pub failed_files: usize,
    pub diagnostics: Vec<String>,
}

/// Public struct `MemorySearchMatch` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemorySearchMatch {
    pub memory_id: String,
    pub score: f32,
    pub vector_score: Option<f32>,
    pub lexical_score: Option<f32>,
    pub fused_score: Option<f32>,
    pub graph_score: Option<f32>,
    pub scope: MemoryScope,
    pub summary: String,
    pub memory_type: MemoryType,
    pub importance: f32,
    pub tags: Vec<String>,
    pub facts: Vec<String>,
    pub source_event_key: String,
    pub embedding_source: String,
    pub embedding_model: Option<String>,
    pub relations: Vec<MemoryRelation>,
}

/// Public struct `MemorySearchResult` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemorySearchResult {
    pub query: String,
    pub scanned: usize,
    pub returned: usize,
    pub retrieval_backend: String,
    pub retrieval_reason_code: String,
    pub embedding_backend: String,
    pub embedding_reason_code: String,
    pub migrated_entries: usize,
    pub query_embedding_latency_ms: u64,
    pub ranking_latency_ms: u64,
    pub lexical_ranking_latency_ms: u64,
    pub fusion_latency_ms: u64,
    pub vector_candidates: usize,
    pub lexical_candidates: usize,
    pub baseline_hash_overlap: Option<usize>,
    pub baseline_vector_overlap: Option<usize>,
    pub matches: Vec<MemorySearchMatch>,
}

/// Public struct `MemoryTreeNode` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryTreeNode {
    pub id: String,
    pub level: String,
    pub entry_count: usize,
    pub children: Vec<MemoryTreeNode>,
}

/// Public struct `MemoryTree` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryTree {
    pub total_entries: usize,
    pub workspaces: Vec<MemoryTreeNode>,
}

/// Public struct `RankedTextCandidate` used across Tau components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedTextCandidate {
    pub key: String,
    pub text: String,
}

/// Public struct `RankedTextMatch` used across Tau components.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedTextMatch {
    pub key: String,
    pub text: String,
    pub score: f32,
}
