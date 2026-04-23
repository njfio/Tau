//! `FileMemoryStore` — the file/SQLite-backed implementation of the memory
//! runtime surface. Extracted from `runtime.rs` as increment 2 of the
//! god-file split documented in
//! `docs/planning/god-file-split-audit-2026-04-23.md`. No behavioral change.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use rusqlite::params;

use crate::memory_contract::{MemoryEntry, MemoryScope};

use super::backend::{
    append_record_jsonl, append_record_sqlite, initialize_memory_sqlite_schema, load_records_jsonl,
    load_records_sqlite, load_relation_map_sqlite, open_memory_sqlite_connection,
    resolve_memory_backend,
};
use super::normalize::{
    current_unix_timestamp_ms, normalize_entry, normalize_relations, normalize_scope,
    sqlite_i64_from_u64, sqlite_u64_from_i64,
};
use super::ranking::record_search_text_for_entry;
use super::{
    MemoryEmbeddingProviderConfig, MemoryRelation, MemoryRelationInput, MemoryScopeFilter, MemoryStorageBackend, MemoryType, MemoryTypeImportanceProfile, MemoryWriteResult,
    RuntimeMemoryRecord,
    MEMORY_RUNTIME_ENTRIES_FILE_NAME,
    MEMORY_RUNTIME_SCHEMA_VERSION, MEMORY_STORAGE_REASON_INIT_IMPORT_FAILED,
};

/// Public struct `FileMemoryStore` used across Tau components.
#[derive(Debug, Clone, PartialEq)]
pub struct FileMemoryStore {
    root_dir: PathBuf,
    pub(super) embedding_provider: Option<MemoryEmbeddingProviderConfig>,
    default_importance_profile: MemoryTypeImportanceProfile,
    pub(super) storage_backend: MemoryStorageBackend,
    storage_path: Option<PathBuf>,
    backend_reason_code: String,
    backend_init_error: Option<String>,
}

impl FileMemoryStore {
    /// Creates a file-backed store rooted at `root_dir`.
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self::new_with_embedding_provider_and_importance_profile(root_dir, None, None)
    }

    /// Creates a file-backed store rooted at `root_dir` with optional embedding provider config.
    pub fn new_with_embedding_provider(
        root_dir: impl Into<PathBuf>,
        embedding_provider: Option<MemoryEmbeddingProviderConfig>,
    ) -> Self {
        Self::new_with_embedding_provider_and_importance_profile(root_dir, embedding_provider, None)
    }

    /// Creates a file-backed store rooted at `root_dir` with optional embedding provider and
    /// optional runtime-configured memory-type default importance profile.
    pub fn new_with_embedding_provider_and_importance_profile(
        root_dir: impl Into<PathBuf>,
        embedding_provider: Option<MemoryEmbeddingProviderConfig>,
        default_importance_profile: Option<MemoryTypeImportanceProfile>,
    ) -> Self {
        let root_dir = root_dir.into();
        let resolved = resolve_memory_backend(&root_dir);
        let mut profile = default_importance_profile.unwrap_or_default();
        if let Err(error) = profile.validate() {
            tracing::warn!(
                error = %error,
                "invalid runtime memory type default importance profile; falling back to built-in defaults"
            );
            profile = MemoryTypeImportanceProfile::default();
        }
        let mut store = Self {
            root_dir,
            embedding_provider,
            default_importance_profile: profile,
            storage_backend: resolved.backend,
            storage_path: resolved.storage_path,
            backend_reason_code: resolved.reason_code,
            backend_init_error: None,
        };
        if store.storage_backend == MemoryStorageBackend::Sqlite {
            if let Err(error) = store.maybe_import_legacy_jsonl_into_sqlite() {
                store.backend_init_error = Some(error.to_string());
                store.backend_reason_code = MEMORY_STORAGE_REASON_INIT_IMPORT_FAILED.to_string();
            }
        }
        store
    }

    /// Returns the store root directory.
    pub fn root_dir(&self) -> &Path {
        self.root_dir.as_path()
    }

    /// Returns the active storage backend.
    pub fn storage_backend(&self) -> MemoryStorageBackend {
        self.storage_backend
    }

    /// Returns the active storage backend label.
    pub fn storage_backend_label(&self) -> &'static str {
        self.storage_backend.label()
    }

    /// Returns the backend selection reason code.
    pub fn storage_backend_reason_code(&self) -> &str {
        self.backend_reason_code.as_str()
    }

    /// Returns the resolved storage file path, when applicable.
    pub fn storage_path(&self) -> Option<&Path> {
        self.storage_path.as_deref()
    }

    /// Returns the resolved runtime-configured memory-type default importance profile.
    pub fn default_importance_profile(&self) -> &MemoryTypeImportanceProfile {
        &self.default_importance_profile
    }

    /// Imports JSONL artifacts into the active backend.
    pub fn import_jsonl_artifact(&self, source: &Path) -> Result<usize> {
        let records = load_records_jsonl(source)?;
        if records.is_empty() {
            return Ok(0);
        }

        self.ensure_backend_ready()?;
        match self.storage_backend {
            MemoryStorageBackend::Jsonl => {
                for record in &records {
                    append_record_jsonl(self.storage_path_required()?, record)?;
                }
                Ok(records.len())
            }
            MemoryStorageBackend::Sqlite => {
                let mut connection = open_memory_sqlite_connection(self.storage_path_required()?)?;
                initialize_memory_sqlite_schema(&connection)?;
                let transaction = connection.transaction()?;
                for record in &records {
                    let updated_unix_ms =
                        sqlite_i64_from_u64(record.updated_unix_ms, "updated_unix_ms")?;
                    let encoded =
                        serde_json::to_string(record).context("failed to encode memory record")?;
                    transaction.execute(
                        r#"
                        INSERT INTO memory_records (memory_id, updated_unix_ms, record_json)
                        VALUES (?1, ?2, ?3)
                        "#,
                        params![record.entry.memory_id, updated_unix_ms, encoded],
                    )?;
                }
                transaction.commit()?;
                Ok(records.len())
            }
        }
    }

    /// Writes or updates a memory entry under `scope`.
    pub fn write_entry(
        &self,
        scope: &MemoryScope,
        entry: MemoryEntry,
    ) -> Result<MemoryWriteResult> {
        self.write_entry_with_metadata(scope, entry, None, None)
    }

    /// Writes or updates a memory entry with optional typed-memory metadata.
    pub fn write_entry_with_metadata(
        &self,
        scope: &MemoryScope,
        entry: MemoryEntry,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
    ) -> Result<MemoryWriteResult> {
        self.write_entry_with_metadata_and_relations(scope, entry, memory_type, importance, &[])
    }

    /// Writes or updates a memory entry with metadata and explicit relations.
    pub fn write_entry_with_metadata_and_relations(
        &self,
        scope: &MemoryScope,
        entry: MemoryEntry,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
        relations: &[MemoryRelationInput],
    ) -> Result<MemoryWriteResult> {
        let normalized_scope = normalize_scope(scope);
        let normalized_entry = normalize_entry(entry)?;
        let resolved_memory_type = memory_type.unwrap_or_default();
        let resolved_importance = match importance {
            Some(value) if value.is_finite() && (0.0..=1.0).contains(&value) => value,
            Some(value) => {
                bail!("importance must be within 0.0..=1.0 (received {value})")
            }
            None => self
                .default_importance_profile
                .importance_for(resolved_memory_type),
        };
        let existing_records = self.load_latest_records()?;
        let known_memory_ids = existing_records
            .iter()
            .map(|record| record.entry.memory_id.clone())
            .collect::<BTreeSet<_>>();
        let normalized_relations = normalize_relations(
            normalized_entry.memory_id.as_str(),
            relations,
            &known_memory_ids,
        )?;

        let created = existing_records.iter().all(|record| {
            record.entry.memory_id != normalized_entry.memory_id || record.scope != normalized_scope
        });

        let embedding_text = record_search_text_for_entry(&normalized_entry);
        let embedding_dimensions = self
            .embedding_provider
            .as_ref()
            .map(|config| config.dimensions)
            .unwrap_or(128);
        let computed_embedding =
            self.compute_embedding(&embedding_text, embedding_dimensions, true);
        let record = RuntimeMemoryRecord {
            schema_version: MEMORY_RUNTIME_SCHEMA_VERSION,
            updated_unix_ms: current_unix_timestamp_ms(),
            scope: normalized_scope,
            entry: normalized_entry,
            memory_type: resolved_memory_type,
            importance: resolved_importance,
            embedding_source: computed_embedding.backend,
            embedding_model: computed_embedding.model,
            embedding_vector: computed_embedding.vector,
            embedding_reason_code: computed_embedding.reason_code,
            last_accessed_at_unix_ms: 0,
            access_count: 0,
            forgotten: false,
            relations: normalized_relations,
        };
        self.append_record_backend(&record)?;
        Ok(MemoryWriteResult { record, created })
    }

    /// Marks the latest active memory record as forgotten without removing historical data.
    pub fn soft_delete_entry(
        &self,
        memory_id: &str,
        scope_filter: Option<&MemoryScopeFilter>,
    ) -> Result<Option<RuntimeMemoryRecord>> {
        let normalized_memory_id = memory_id.trim();
        if normalized_memory_id.is_empty() {
            bail!("memory_id must not be empty");
        }
        let records = self.load_latest_records_including_forgotten()?;
        let Some(existing) = records.into_iter().find(|record| {
            record.entry.memory_id == normalized_memory_id
                && !record.forgotten
                && scope_filter
                    .map(|filter| filter.matches_scope(&record.scope))
                    .unwrap_or(true)
        }) else {
            return Ok(None);
        };

        let mut forgotten_record = existing;
        forgotten_record.updated_unix_ms = current_unix_timestamp_ms();
        forgotten_record.forgotten = true;
        self.append_record_backend(&forgotten_record)?;
        Ok(Some(forgotten_record))
    }

    pub(super) fn touch_entry_access(
        &self,
        record: &RuntimeMemoryRecord,
    ) -> Result<RuntimeMemoryRecord> {
        let mut touched = record.clone();
        let now_unix_ms = current_unix_timestamp_ms();
        touched.updated_unix_ms = now_unix_ms;
        touched.last_accessed_at_unix_ms = now_unix_ms;
        touched.access_count = touched.access_count.saturating_add(1);
        self.append_record_backend(&touched)?;
        Ok(touched)
    }

    fn ensure_backend_ready(&self) -> Result<()> {
        if let Some(error) = &self.backend_init_error {
            bail!(
                "memory storage backend initialization failed (reason_code={}): {}",
                self.backend_reason_code,
                error
            );
        }
        Ok(())
    }

    pub(super) fn storage_path_required(&self) -> Result<&Path> {
        self.storage_path.as_deref().ok_or_else(|| {
            anyhow!(
                "memory storage backend '{}' does not define a filesystem path",
                self.storage_backend.label()
            )
        })
    }

    pub(super) fn append_record_backend(&self, record: &RuntimeMemoryRecord) -> Result<()> {
        self.ensure_backend_ready()?;
        match self.storage_backend {
            MemoryStorageBackend::Jsonl => {
                append_record_jsonl(self.storage_path_required()?, record)
            }
            MemoryStorageBackend::Sqlite => {
                append_record_sqlite(self.storage_path_required()?, record)
            }
        }
    }

    pub(super) fn load_records_backend(&self) -> Result<Vec<RuntimeMemoryRecord>> {
        self.ensure_backend_ready()?;
        match self.storage_backend {
            MemoryStorageBackend::Jsonl => load_records_jsonl(self.storage_path_required()?),
            MemoryStorageBackend::Sqlite => load_records_sqlite(self.storage_path_required()?),
        }
    }

    pub(super) fn load_relation_map_backend(&self) -> Result<HashMap<String, Vec<MemoryRelation>>> {
        self.ensure_backend_ready()?;
        match self.storage_backend {
            MemoryStorageBackend::Jsonl => Ok(HashMap::new()),
            MemoryStorageBackend::Sqlite => load_relation_map_sqlite(self.storage_path_required()?),
        }
    }

    fn maybe_import_legacy_jsonl_into_sqlite(&self) -> Result<usize> {
        if self.storage_backend != MemoryStorageBackend::Sqlite {
            return Ok(0);
        }
        let Some(sqlite_path) = self.storage_path.as_deref() else {
            return Ok(0);
        };
        let Some(legacy_path) = self.legacy_jsonl_import_path() else {
            return Ok(0);
        };
        if !legacy_path.exists() {
            return Ok(0);
        }

        let mut connection = open_memory_sqlite_connection(sqlite_path)?;
        initialize_memory_sqlite_schema(&connection)?;
        let existing_count_i64: i64 = connection
            .query_row("SELECT COUNT(1) FROM memory_records", [], |row| row.get(0))
            .context("failed to inspect sqlite memory record count")?;
        let existing_count = sqlite_u64_from_i64(existing_count_i64, "memory_records_count")
            .with_context(|| {
                format!(
                    "failed to parse sqlite memory record count from {}",
                    sqlite_path.display()
                )
            })?;
        if existing_count > 0 {
            return Ok(0);
        }

        let records = load_records_jsonl(&legacy_path)?;
        if records.is_empty() {
            return Ok(0);
        }

        let transaction = connection.transaction()?;
        for record in &records {
            let updated_unix_ms = sqlite_i64_from_u64(record.updated_unix_ms, "updated_unix_ms")?;
            let encoded =
                serde_json::to_string(record).context("failed to encode memory sqlite record")?;
            transaction.execute(
                r#"
                INSERT INTO memory_records (memory_id, updated_unix_ms, record_json)
                VALUES (?1, ?2, ?3)
                "#,
                params![record.entry.memory_id, updated_unix_ms, encoded],
            )?;
        }
        transaction.commit()?;
        Ok(records.len())
    }

    fn legacy_jsonl_import_path(&self) -> Option<PathBuf> {
        match self.storage_backend {
            MemoryStorageBackend::Jsonl => None,
            MemoryStorageBackend::Sqlite => {
                if self.root_dir.extension().and_then(|value| value.to_str()) == Some("sqlite")
                    || self.root_dir.extension().and_then(|value| value.to_str()) == Some("db")
                {
                    let legacy = self.root_dir.with_extension("jsonl");
                    if self.storage_path.as_deref() == Some(legacy.as_path()) {
                        None
                    } else {
                        Some(legacy)
                    }
                } else {
                    let legacy = self.root_dir.join(MEMORY_RUNTIME_ENTRIES_FILE_NAME);
                    if self.storage_path.as_deref() == Some(legacy.as_path()) {
                        None
                    } else {
                        Some(legacy)
                    }
                }
            }
        }
    }
}
