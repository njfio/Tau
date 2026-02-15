use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::memory_contract::{MemoryEntry, MemoryScope};

const MEMORY_RUNTIME_SCHEMA_VERSION: u32 = 1;
const MEMORY_RUNTIME_ENTRIES_FILE_NAME: &str = "entries.jsonl";
const MEMORY_SCOPE_DEFAULT_WORKSPACE: &str = "default-workspace";
const MEMORY_SCOPE_DEFAULT_CHANNEL: &str = "default-channel";
const MEMORY_SCOPE_DEFAULT_ACTOR: &str = "default-actor";

/// Public struct `RuntimeMemoryRecord` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeMemoryRecord {
    pub schema_version: u32,
    pub updated_unix_ms: u64,
    pub scope: MemoryScope,
    pub entry: MemoryEntry,
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl Default for MemorySearchOptions {
    fn default() -> Self {
        Self {
            scope: MemoryScopeFilter::default(),
            limit: 5,
            embedding_dimensions: 128,
            min_similarity: 0.55,
        }
    }
}

/// Public struct `MemorySearchMatch` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemorySearchMatch {
    pub memory_id: String,
    pub score: f32,
    pub scope: MemoryScope,
    pub summary: String,
    pub tags: Vec<String>,
    pub facts: Vec<String>,
    pub source_event_key: String,
}

/// Public struct `MemorySearchResult` used across Tau components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemorySearchResult {
    pub query: String,
    pub scanned: usize,
    pub returned: usize,
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

/// Public struct `FileMemoryStore` used across Tau components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMemoryStore {
    root_dir: PathBuf,
}

impl FileMemoryStore {
    /// Creates a file-backed store rooted at `root_dir`.
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    /// Returns the store root directory.
    pub fn root_dir(&self) -> &Path {
        self.root_dir.as_path()
    }

    /// Writes or updates a memory entry under `scope`.
    pub fn write_entry(
        &self,
        scope: &MemoryScope,
        entry: MemoryEntry,
    ) -> Result<MemoryWriteResult> {
        let normalized_scope = normalize_scope(scope);
        let normalized_entry = normalize_entry(entry)?;

        let created = self
            .read_entry(
                normalized_entry.memory_id.as_str(),
                Some(&MemoryScopeFilter {
                    workspace_id: Some(normalized_scope.workspace_id.clone()),
                    channel_id: Some(normalized_scope.channel_id.clone()),
                    actor_id: Some(normalized_scope.actor_id.clone()),
                }),
            )?
            .is_none();

        let record = RuntimeMemoryRecord {
            schema_version: MEMORY_RUNTIME_SCHEMA_VERSION,
            updated_unix_ms: current_unix_timestamp_ms(),
            scope: normalized_scope,
            entry: normalized_entry,
        };
        append_record(self.entries_path().as_path(), &record)?;
        Ok(MemoryWriteResult { record, created })
    }

    /// Reads the latest record for `memory_id`, optionally constrained by `scope_filter`.
    pub fn read_entry(
        &self,
        memory_id: &str,
        scope_filter: Option<&MemoryScopeFilter>,
    ) -> Result<Option<RuntimeMemoryRecord>> {
        let normalized_memory_id = memory_id.trim();
        if normalized_memory_id.is_empty() {
            bail!("memory_id must not be empty");
        }
        let records = self.load_latest_records()?;
        Ok(records.into_iter().find(|record| {
            record.entry.memory_id == normalized_memory_id
                && scope_filter
                    .map(|filter| filter.matches_scope(&record.scope))
                    .unwrap_or(true)
        }))
    }

    /// Returns latest records filtered by scope and bounded by `limit`.
    pub fn list_latest_records(
        &self,
        scope_filter: Option<&MemoryScopeFilter>,
        limit: usize,
    ) -> Result<Vec<RuntimeMemoryRecord>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut records = self.load_latest_records()?;
        if let Some(filter) = scope_filter {
            records.retain(|record| filter.matches_scope(&record.scope));
        }
        records.truncate(limit);
        Ok(records)
    }

    /// Performs deterministic semantic search over latest records.
    pub fn search(&self, query: &str, options: &MemorySearchOptions) -> Result<MemorySearchResult> {
        let normalized_query = query.trim();
        if normalized_query.is_empty() {
            bail!("query must not be empty");
        }

        let records = self.list_latest_records(Some(&options.scope), usize::MAX)?;
        let candidates = records
            .iter()
            .map(|record| RankedTextCandidate {
                key: record.entry.memory_id.clone(),
                text: record_search_text(record),
            })
            .collect::<Vec<_>>();

        let ranked = rank_text_candidates(
            normalized_query,
            candidates,
            options.limit,
            options.embedding_dimensions,
            options.min_similarity,
        );
        let by_memory_id = records
            .into_iter()
            .map(|record| (record.entry.memory_id.clone(), record))
            .collect::<HashMap<_, _>>();

        let mut matches = Vec::with_capacity(ranked.len());
        for item in ranked {
            let Some(record) = by_memory_id.get(&item.key) else {
                continue;
            };
            matches.push(MemorySearchMatch {
                memory_id: record.entry.memory_id.clone(),
                score: item.score,
                scope: record.scope.clone(),
                summary: record.entry.summary.clone(),
                tags: record.entry.tags.clone(),
                facts: record.entry.facts.clone(),
                source_event_key: record.entry.source_event_key.clone(),
            });
        }

        Ok(MemorySearchResult {
            query: normalized_query.to_string(),
            scanned: by_memory_id.len(),
            returned: matches.len(),
            matches,
        })
    }

    /// Returns a hierarchical workspace/channel/actor tree for latest records.
    pub fn tree(&self) -> Result<MemoryTree> {
        let records = self.load_latest_records()?;
        let mut by_scope = BTreeMap::<String, BTreeMap<String, BTreeMap<String, usize>>>::new();

        for record in records {
            let workspace = record.scope.workspace_id;
            let channel = record.scope.channel_id;
            let actor = record.scope.actor_id;
            *by_scope
                .entry(workspace)
                .or_default()
                .entry(channel)
                .or_default()
                .entry(actor)
                .or_default() += 1;
        }

        let mut total_entries = 0usize;
        let mut workspaces = Vec::with_capacity(by_scope.len());
        for (workspace_id, channels) in by_scope {
            let mut workspace_count = 0usize;
            let mut channel_nodes = Vec::with_capacity(channels.len());
            for (channel_id, actors) in channels {
                let mut channel_count = 0usize;
                let mut actor_nodes = Vec::with_capacity(actors.len());
                for (actor_id, actor_count) in actors {
                    channel_count = channel_count.saturating_add(actor_count);
                    actor_nodes.push(MemoryTreeNode {
                        id: actor_id,
                        level: "actor".to_string(),
                        entry_count: actor_count,
                        children: Vec::new(),
                    });
                }
                workspace_count = workspace_count.saturating_add(channel_count);
                channel_nodes.push(MemoryTreeNode {
                    id: channel_id,
                    level: "channel".to_string(),
                    entry_count: channel_count,
                    children: actor_nodes,
                });
            }
            total_entries = total_entries.saturating_add(workspace_count);
            workspaces.push(MemoryTreeNode {
                id: workspace_id,
                level: "workspace".to_string(),
                entry_count: workspace_count,
                children: channel_nodes,
            });
        }

        Ok(MemoryTree {
            total_entries,
            workspaces,
        })
    }

    fn load_latest_records(&self) -> Result<Vec<RuntimeMemoryRecord>> {
        let records = load_records(self.entries_path().as_path())?;
        let mut seen = BTreeSet::new();
        let mut latest = Vec::new();
        for record in records.into_iter().rev() {
            if seen.insert(record.entry.memory_id.clone()) {
                latest.push(record);
            }
        }
        latest.sort_by(|left, right| {
            right
                .updated_unix_ms
                .cmp(&left.updated_unix_ms)
                .then_with(|| left.entry.memory_id.cmp(&right.entry.memory_id))
        });
        Ok(latest)
    }

    fn entries_path(&self) -> PathBuf {
        self.root_dir.join(MEMORY_RUNTIME_ENTRIES_FILE_NAME)
    }
}

/// Ranks text candidates using deterministic hash embeddings and cosine similarity.
pub fn rank_text_candidates(
    query: &str,
    candidates: Vec<RankedTextCandidate>,
    limit: usize,
    dimensions: usize,
    min_similarity: f32,
) -> Vec<RankedTextMatch> {
    if limit == 0 {
        return Vec::new();
    }
    let normalized_query = query.trim();
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let query_embedding = embed_text_vector(normalized_query, dimensions);
    if query_embedding.iter().all(|component| *component == 0.0) {
        return Vec::new();
    }

    let mut matches = candidates
        .into_iter()
        .filter_map(|candidate| {
            let candidate_embedding = embed_text_vector(candidate.text.as_str(), dimensions);
            let score = cosine_similarity(&query_embedding, &candidate_embedding);
            if score >= min_similarity {
                Some(RankedTextMatch {
                    key: candidate.key,
                    text: candidate.text,
                    score,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.key.cmp(&right.key))
    });
    matches.truncate(limit);
    matches
}

/// Converts text to a normalized fixed-size vector using FNV-1a token hashing.
pub fn embed_text_vector(text: &str, dimensions: usize) -> Vec<f32> {
    let dimensions = dimensions.max(1);
    let mut vector = vec![0.0f32; dimensions];
    for raw_token in text.split(|character: char| !character.is_alphanumeric()) {
        if raw_token.is_empty() {
            continue;
        }
        let token = raw_token.to_ascii_lowercase();
        let hash = fnv1a_hash(token.as_bytes());
        let index = (hash as usize) % dimensions;
        let sign = if (hash & 1) == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
    }

    let magnitude = vector
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    if magnitude > 0.0 {
        for component in &mut vector {
            *component /= magnitude;
        }
    }
    vector
}

/// Computes cosine similarity for equal-length vectors.
pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() {
        return 0.0;
    }
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn fnv1a_hash(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn record_search_text(record: &RuntimeMemoryRecord) -> String {
    let mut parts = Vec::with_capacity(3);
    parts.push(record.entry.summary.clone());
    if !record.entry.tags.is_empty() {
        parts.push(record.entry.tags.join(" "));
    }
    if !record.entry.facts.is_empty() {
        parts.push(record.entry.facts.join(" "));
    }
    parts.join("\n")
}

fn normalize_scope(scope: &MemoryScope) -> MemoryScope {
    MemoryScope {
        workspace_id: normalize_scope_component(
            &scope.workspace_id,
            MEMORY_SCOPE_DEFAULT_WORKSPACE,
        ),
        channel_id: normalize_scope_component(&scope.channel_id, MEMORY_SCOPE_DEFAULT_CHANNEL),
        actor_id: normalize_scope_component(&scope.actor_id, MEMORY_SCOPE_DEFAULT_ACTOR),
    }
}

fn normalize_scope_component(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_entry(entry: MemoryEntry) -> Result<MemoryEntry> {
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

fn append_record(path: &Path, record: &RuntimeMemoryRecord) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create memory store root {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open memory entries file {}", path.display()))?;
    let encoded = serde_json::to_string(record).context("failed to encode memory record")?;
    file.write_all(encoded.as_bytes())
        .with_context(|| format!("failed to write memory record to {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("failed to write newline to {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush memory entries file {}", path.display()))?;
    Ok(())
}

fn load_records(path: &Path) -> Result<Vec<RuntimeMemoryRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path)
        .with_context(|| format!("failed to open memory entries file {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "failed to read memory entries file {} at line {}",
                path.display(),
                index + 1
            )
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record = serde_json::from_str::<RuntimeMemoryRecord>(trimmed).with_context(|| {
            format!(
                "failed to parse memory entries file {} at line {}",
                path.display(),
                index + 1
            )
        })?;
        records.push(record);
    }
    Ok(records)
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        embed_text_vector, rank_text_candidates, FileMemoryStore, MemoryScopeFilter,
        MemorySearchOptions, RankedTextCandidate,
    };
    use crate::memory_contract::{MemoryEntry, MemoryScope};
    use tempfile::tempdir;

    #[test]
    fn unit_embed_text_vector_normalizes_non_empty_inputs() {
        let vector = embed_text_vector("release checklist", 32);
        let magnitude = vector
            .iter()
            .map(|component| component * component)
            .sum::<f32>()
            .sqrt();
        assert!(magnitude > 0.99);
        assert!(magnitude <= 1.001);
    }

    #[test]
    fn functional_file_memory_store_write_and_read_round_trips_latest_record() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "channel-1".to_string(),
            actor_id: "assistant".to_string(),
        };

        let first = MemoryEntry {
            memory_id: "memory-1".to_string(),
            summary: "remember release checklist owner".to_string(),
            tags: vec!["release".to_string()],
            facts: vec!["owner=ops".to_string()],
            source_event_key: "evt-1".to_string(),
            recency_weight_bps: 120,
            confidence_bps: 880,
        };
        let second = MemoryEntry {
            summary: "remember release checklist owner + rollout order".to_string(),
            source_event_key: "evt-2".to_string(),
            ..first.clone()
        };

        let first_result = store.write_entry(&scope, first).expect("first write");
        assert!(first_result.created);
        let second_result = store.write_entry(&scope, second).expect("second write");
        assert!(!second_result.created);

        let loaded = store
            .read_entry("memory-1", None)
            .expect("read")
            .expect("existing");
        assert_eq!(
            loaded.entry.summary,
            "remember release checklist owner + rollout order"
        );
        assert_eq!(loaded.entry.source_event_key, "evt-2");
    }

    #[test]
    fn integration_memory_search_uses_ranked_candidates_with_scope_filter() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope_a = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "deploy".to_string(),
            actor_id: "assistant".to_string(),
        };
        let scope_b = MemoryScope {
            workspace_id: "workspace-b".to_string(),
            channel_id: "support".to_string(),
            actor_id: "assistant".to_string(),
        };

        store
            .write_entry(
                &scope_a,
                MemoryEntry {
                    memory_id: "memory-release".to_string(),
                    summary: "Nightly release checklist requires smoke tests".to_string(),
                    tags: vec!["release".to_string(), "nightly".to_string()],
                    facts: vec!["run smoke".to_string()],
                    source_event_key: "evt-a".to_string(),
                    recency_weight_bps: 90,
                    confidence_bps: 820,
                },
            )
            .expect("write release memory");
        store
            .write_entry(
                &scope_b,
                MemoryEntry {
                    memory_id: "memory-support".to_string(),
                    summary: "Support rotation uses weekend escalation".to_string(),
                    tags: vec!["support".to_string()],
                    facts: vec!["pager escalation".to_string()],
                    source_event_key: "evt-b".to_string(),
                    recency_weight_bps: 70,
                    confidence_bps: 700,
                },
            )
            .expect("write support memory");

        let result = store
            .search(
                "release smoke checklist",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter {
                        workspace_id: Some("workspace-a".to_string()),
                        channel_id: None,
                        actor_id: None,
                    },
                    limit: 5,
                    embedding_dimensions: 128,
                    min_similarity: 0.1,
                },
            )
            .expect("search");
        assert_eq!(result.returned, 1);
        assert_eq!(result.matches[0].memory_id, "memory-release");
        assert_eq!(result.matches[0].scope.workspace_id, "workspace-a");
    }

    #[test]
    fn regression_memory_tree_counts_latest_entry_versions_once() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "deploy".to_string(),
            actor_id: "assistant".to_string(),
        };

        let first = MemoryEntry {
            memory_id: "memory-1".to_string(),
            summary: "first".to_string(),
            tags: Vec::new(),
            facts: Vec::new(),
            source_event_key: "evt-1".to_string(),
            recency_weight_bps: 0,
            confidence_bps: 0,
        };
        store
            .write_entry(&scope, first.clone())
            .expect("write first version");
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    summary: "second".to_string(),
                    source_event_key: "evt-2".to_string(),
                    ..first
                },
            )
            .expect("write second version");

        let tree = store.tree().expect("tree");
        assert_eq!(tree.total_entries, 1);
        assert_eq!(tree.workspaces.len(), 1);
        assert_eq!(tree.workspaces[0].entry_count, 1);
        assert_eq!(tree.workspaces[0].children[0].entry_count, 1);
        assert_eq!(tree.workspaces[0].children[0].children[0].entry_count, 1);
    }

    #[test]
    fn unit_rank_text_candidates_returns_highest_similarity_first() {
        let ranked = rank_text_candidates(
            "release checklist",
            vec![
                RankedTextCandidate {
                    key: "a".to_string(),
                    text: "release checklist smoke tests".to_string(),
                },
                RankedTextCandidate {
                    key: "b".to_string(),
                    text: "team lunch planning".to_string(),
                },
            ],
            2,
            128,
            0.1,
        );
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].key, "a");
    }
}
