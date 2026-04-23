

use crate::memory_contract::MemoryEntry;

mod backend;
mod defaults;
mod file_store;
mod normalize;
mod query;
mod ranking;

pub use file_store::FileMemoryStore;

#[cfg(test)]
use backend::{initialize_memory_sqlite_schema, open_memory_sqlite_connection};
use normalize::current_unix_timestamp_ms;
#[cfg(test)]
use normalize::{normalize_relations, sqlite_i64_from_u64, sqlite_u64_from_i64};
pub use ranking::{
    cosine_similarity, embed_text_vector, rank_text_candidates, rank_text_candidates_bm25,
};
use ranking::{
    reciprocal_rank_fuse, record_search_text,
    resize_and_normalize_embedding,
};

const MEMORY_RUNTIME_SCHEMA_VERSION: u32 = 1;
const MEMORY_RUNTIME_ENTRIES_FILE_NAME: &str = "entries.jsonl";
const MEMORY_RUNTIME_ENTRIES_SQLITE_FILE_NAME: &str = "entries.sqlite";
const MEMORY_BACKEND_ENV: &str = "TAU_MEMORY_BACKEND";
const MEMORY_SCOPE_DEFAULT_WORKSPACE: &str = "default-workspace";
const MEMORY_SCOPE_DEFAULT_CHANNEL: &str = "default-channel";
const MEMORY_SCOPE_DEFAULT_ACTOR: &str = "default-actor";
const MEMORY_EMBEDDING_SOURCE_HASH: &str = "hash-fnv1a";
const MEMORY_EMBEDDING_SOURCE_PROVIDER: &str = "provider-openai-compatible";
const MEMORY_EMBEDDING_SOURCE_PROVIDER_LOCAL_FASTEMBED: &str = "provider-local-fastembed";
const MEMORY_EMBEDDING_REASON_HASH_ONLY: &str = "memory_embedding_hash_only";
const MEMORY_EMBEDDING_REASON_PROVIDER_SUCCESS: &str = "memory_embedding_provider_success";
const MEMORY_EMBEDDING_REASON_PROVIDER_FAILED: &str = "memory_embedding_provider_failed";
const MEMORY_RETRIEVAL_BACKEND_VECTOR_ONLY: &str = "vector-only";
const MEMORY_RETRIEVAL_BACKEND_HYBRID_BM25_RRF: &str = "hybrid-bm25-rrf";
const MEMORY_RETRIEVAL_REASON_VECTOR_ONLY: &str = "memory_retrieval_vector_only";
const MEMORY_RETRIEVAL_REASON_HYBRID_ENABLED: &str = "memory_retrieval_hybrid_enabled";
const MEMORY_LIFECYCLE_DEFAULT_STALE_AFTER_MS: u64 = 7 * 24 * 60 * 60 * 1_000;
const MEMORY_LIFECYCLE_DEFAULT_DECAY_RATE: f32 = 0.9;
const MEMORY_LIFECYCLE_DEFAULT_PRUNE_IMPORTANCE_FLOOR: f32 = 0.1;
const MEMORY_LIFECYCLE_DEFAULT_ORPHAN_IMPORTANCE_FLOOR: f32 = 0.2;
const MEMORY_LIFECYCLE_DEFAULT_DUPLICATE_SIMILARITY_THRESHOLD: f32 = 0.97;
const MEMORY_INGESTION_DEFAULT_CHUNK_LINE_COUNT: usize = 200;
const MEMORY_INGESTION_LLM_DEFAULT_TIMEOUT_MS: u64 = 15_000;
const MEMORY_STORAGE_REASON_PATH_JSONL: &str = "memory_storage_backend_path_jsonl";
const MEMORY_STORAGE_REASON_PATH_SQLITE: &str = "memory_storage_backend_path_sqlite";
const MEMORY_STORAGE_REASON_EXISTING_JSONL: &str = "memory_storage_backend_existing_jsonl";
const MEMORY_STORAGE_REASON_EXISTING_SQLITE: &str = "memory_storage_backend_existing_sqlite";
const MEMORY_STORAGE_REASON_DEFAULT_SQLITE: &str = "memory_storage_backend_default_sqlite";
const MEMORY_STORAGE_REASON_ENV_JSONL: &str = "memory_storage_backend_env_jsonl";
const MEMORY_STORAGE_REASON_ENV_SQLITE: &str = "memory_storage_backend_env_sqlite";
const MEMORY_STORAGE_REASON_ENV_AUTO: &str = "memory_storage_backend_env_auto";
const MEMORY_STORAGE_REASON_ENV_INVALID_FALLBACK: &str =
    "memory_storage_backend_env_invalid_fallback";
const MEMORY_STORAGE_REASON_INIT_IMPORT_FAILED: &str = "memory_storage_backend_import_failed";
const MEMORY_GRAPH_SIGNAL_WEIGHT_DEFAULT: f32 = 0.25;
pub const MEMORY_INVALID_RELATION_REASON_CODE: &str = "memory_invalid_relation";

mod types;
pub use types::*;
use types::{ComputedEmbedding, ResolvedMemoryBackend};


#[cfg(test)]
mod tests;
