//! Store/provider/ingestion tests for the memory runtime.
//!
//! Extracted from `runtime/tests.rs` per the 2026-04-23 split audit
//! (Proposal 3, increment 5d). Covers FileMemoryStore backend
//! resolution, SQLite/JSONL backend import, round-trip, provider
//! embedding metadata persistence, and the spec_2553 remote/local
//! provider path regressions.

use super::super::ranking::{with_local_embedding_test_mode, LocalEmbeddingTestMode};
use super::super::{
    FileMemoryStore, MemoryEmbeddingProviderConfig, MemoryStorageBackend, MEMORY_BACKEND_ENV,
    MEMORY_STORAGE_REASON_ENV_INVALID_FALLBACK,
};
use super::{memory_backend_env_lock, ScopedEnvVar};
use crate::memory_contract::{MemoryEntry, MemoryScope};
use httpmock::{Method::POST, MockServer};
use tempfile::tempdir;

#[test]
fn functional_memory_store_defaults_to_sqlite_backend_for_directory_roots() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path().join(".tau/memory"));
    assert_eq!(store.storage_backend(), MemoryStorageBackend::Sqlite);
    assert_eq!(
        store.storage_backend_reason_code(),
        "memory_storage_backend_default_sqlite"
    );
    assert!(store
        .storage_path()
        .expect("sqlite storage path")
        .ends_with("entries.sqlite"));
}

#[test]
fn regression_memory_store_treats_postgres_env_backend_as_invalid_and_falls_back() {
    let _guard = memory_backend_env_lock()
        .lock()
        .expect("memory backend env lock");
    let _backend_env = ScopedEnvVar::set(MEMORY_BACKEND_ENV, "postgres");

    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path().join(".tau/memory"));
    assert_eq!(store.storage_backend(), MemoryStorageBackend::Sqlite);
    assert_eq!(
        store.storage_backend_reason_code(),
        MEMORY_STORAGE_REASON_ENV_INVALID_FALLBACK
    );
}

#[test]
fn integration_memory_store_imports_legacy_jsonl_into_sqlite() {
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join(".tau/memory");
    let legacy_jsonl = root.join("entries.jsonl");
    let legacy_store = FileMemoryStore::new_with_embedding_provider(legacy_jsonl.clone(), None);
    let scope = MemoryScope {
        workspace_id: "workspace".to_string(),
        channel_id: "channel".to_string(),
        actor_id: "assistant".to_string(),
    };
    let entry = MemoryEntry {
        memory_id: "memory-legacy".to_string(),
        summary: "legacy-jsonl-entry".to_string(),
        tags: vec!["legacy".to_string()],
        facts: vec!["imported=true".to_string()],
        source_event_key: "evt-legacy".to_string(),
        recency_weight_bps: 0,
        confidence_bps: 1_000,
    };
    legacy_store
        .write_entry(&scope, entry)
        .expect("seed legacy jsonl");

    let sqlite_store = FileMemoryStore::new_with_embedding_provider(root.clone(), None);
    assert_eq!(sqlite_store.storage_backend(), MemoryStorageBackend::Sqlite);
    assert_eq!(
        sqlite_store.storage_backend_reason_code(),
        "memory_storage_backend_existing_jsonl"
    );
    let loaded = sqlite_store
        .read_entry("memory-legacy", None)
        .expect("read legacy")
        .expect("legacy should import");
    assert_eq!(loaded.entry.summary, "legacy-jsonl-entry");
    assert!(root.join("entries.sqlite").exists());
    assert!(legacy_jsonl.exists());
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
fn functional_memory_store_persists_provider_embedding_metadata() {
    let server = MockServer::start();
    let embeddings = server.mock(|when, then| {
        when.method(POST).path("/embeddings");
        then.status(200).json_body_obj(&serde_json::json!({
            "data": [
                { "embedding": [0.4, 0.1, -0.3, 0.2] }
            ]
        }));
    });

    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new_with_embedding_provider(
        temp.path(),
        Some(MemoryEmbeddingProviderConfig {
            provider: "openai-compatible".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_base: server.url(""),
            api_key: "test-key".to_string(),
            dimensions: 8,
            timeout_ms: 5_000,
        }),
    );
    let scope = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "deploy".to_string(),
        actor_id: "assistant".to_string(),
    };
    let write_result = store
        .write_entry(
            &scope,
            MemoryEntry {
                memory_id: "memory-provider".to_string(),
                summary: "release checklist with provider embeddings".to_string(),
                tags: vec!["release".to_string()],
                facts: vec!["owner=ops".to_string()],
                source_event_key: "evt-provider".to_string(),
                recency_weight_bps: 100,
                confidence_bps: 900,
            },
        )
        .expect("provider-backed write");

    embeddings.assert();
    assert_eq!(
        write_result.record.embedding_source,
        "provider-openai-compatible"
    );
    assert_eq!(
        write_result.record.embedding_model,
        Some("text-embedding-3-small".to_string())
    );
    assert_eq!(
        write_result.record.embedding_reason_code,
        "memory_embedding_provider_success"
    );
    assert_eq!(write_result.record.embedding_vector.len(), 8);
    assert!(write_result
        .record
        .embedding_vector
        .iter()
        .any(|value| *value != 0.0));
}

#[test]
fn integration_spec_2553_c02_memory_write_local_provider_success_records_local_embedding_metadata(
) {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new_with_embedding_provider(
        temp.path(),
        Some(MemoryEmbeddingProviderConfig {
            provider: "local".to_string(),
            model: "BAAI/bge-small-en-v1.5".to_string(),
            api_base: String::new(),
            api_key: String::new(),
            dimensions: 8,
            timeout_ms: 5_000,
        }),
    );
    let scope = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "deploy".to_string(),
        actor_id: "assistant".to_string(),
    };
    let write_result = with_local_embedding_test_mode(LocalEmbeddingTestMode::Success, || {
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-local-provider-success".to_string(),
                    summary: "local provider should emit non-hash embedding metadata"
                        .to_string(),
                    tags: vec!["local".to_string()],
                    facts: vec!["provider=local".to_string()],
                    source_event_key: "evt-local-success".to_string(),
                    recency_weight_bps: 100,
                    confidence_bps: 900,
                },
            )
            .expect("local provider write")
    });

    assert_eq!(
        write_result.record.embedding_source,
        "provider-local-fastembed"
    );
    assert_eq!(
        write_result.record.embedding_model,
        Some("BAAI/bge-small-en-v1.5".to_string())
    );
    assert_eq!(
        write_result.record.embedding_reason_code,
        "memory_embedding_provider_success"
    );
    assert_eq!(write_result.record.embedding_vector.len(), 8);
    assert!(write_result
        .record
        .embedding_vector
        .iter()
        .any(|value| *value != 0.0));
}

#[test]
fn regression_spec_2553_c03_memory_write_local_provider_failure_falls_back_to_hash_embedding() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new_with_embedding_provider(
        temp.path(),
        Some(MemoryEmbeddingProviderConfig {
            provider: "local".to_string(),
            model: "unsupported/local-model".to_string(),
            api_base: String::new(),
            api_key: String::new(),
            dimensions: 16,
            timeout_ms: 5_000,
        }),
    );
    let scope = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "deploy".to_string(),
        actor_id: "assistant".to_string(),
    };
    let result = with_local_embedding_test_mode(LocalEmbeddingTestMode::Failure, || {
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-local-provider-fallback".to_string(),
                    summary: "fallback should keep local provider writes online".to_string(),
                    tags: vec!["incident".to_string()],
                    facts: vec!["provider=local".to_string()],
                    source_event_key: "evt-local-fallback".to_string(),
                    recency_weight_bps: 100,
                    confidence_bps: 900,
                },
            )
            .expect("local provider fallback write")
    });

    assert_eq!(result.record.embedding_source, "hash-fnv1a");
    assert_eq!(result.record.embedding_model, None);
    assert_eq!(
        result.record.embedding_reason_code,
        "memory_embedding_provider_failed"
    );
    assert_eq!(result.record.embedding_vector.len(), 16);
}

#[test]
fn regression_spec_2553_c04_remote_embedding_provider_path_preserves_existing_semantics() {
    let server = MockServer::start();
    let success = server.mock(|when, then| {
        when.method(POST)
            .path("/embeddings")
            .body_includes("remote provider success");
        then.status(200).json_body_obj(&serde_json::json!({
            "data": [
                { "embedding": [0.9, 0.1, 0.0, 0.0] }
            ]
        }));
    });
    let failure = server.mock(|when, then| {
        when.method(POST)
            .path("/embeddings")
            .body_includes("remote provider failure");
        then.status(500).json_body_obj(&serde_json::json!({
            "error": "provider outage"
        }));
    });

    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new_with_embedding_provider(
        temp.path(),
        Some(MemoryEmbeddingProviderConfig {
            provider: "openai-compatible".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_base: server.url(""),
            api_key: "test-key".to_string(),
            dimensions: 8,
            timeout_ms: 5_000,
        }),
    );
    let scope = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "deploy".to_string(),
        actor_id: "assistant".to_string(),
    };

    let success_result = store
        .write_entry(
            &scope,
            MemoryEntry {
                memory_id: "memory-remote-success".to_string(),
                summary: "remote provider success".to_string(),
                tags: vec!["release".to_string()],
                facts: vec!["mode=remote".to_string()],
                source_event_key: "evt-remote-success".to_string(),
                recency_weight_bps: 100,
                confidence_bps: 900,
            },
        )
        .expect("remote provider success write");
    let failure_result = store
        .write_entry(
            &scope,
            MemoryEntry {
                memory_id: "memory-remote-failure".to_string(),
                summary: "remote provider failure".to_string(),
                tags: vec!["incident".to_string()],
                facts: vec!["mode=remote".to_string()],
                source_event_key: "evt-remote-failure".to_string(),
                recency_weight_bps: 100,
                confidence_bps: 900,
            },
        )
        .expect("remote provider fallback write");

    success.assert();
    failure.assert();
    assert_eq!(
        success_result.record.embedding_source,
        "provider-openai-compatible"
    );
    assert_eq!(
        success_result.record.embedding_reason_code,
        "memory_embedding_provider_success"
    );
    assert_eq!(
        success_result.record.embedding_model,
        Some("text-embedding-3-small".to_string())
    );
    assert_eq!(failure_result.record.embedding_source, "hash-fnv1a");
    assert_eq!(
        failure_result.record.embedding_reason_code,
        "memory_embedding_provider_failed"
    );
    assert_eq!(failure_result.record.embedding_model, None);
}

#[test]
fn regression_memory_store_falls_back_to_hash_embeddings_on_provider_failure() {
    let server = MockServer::start();
    let _embeddings = server.mock(|when, then| {
        when.method(POST).path("/embeddings");
        then.status(500).json_body_obj(&serde_json::json!({
            "error": "provider outage"
        }));
    });

    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new_with_embedding_provider(
        temp.path(),
        Some(MemoryEmbeddingProviderConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_base: server.url(""),
            api_key: "test-key".to_string(),
            dimensions: 16,
            timeout_ms: 5_000,
        }),
    );
    let scope = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "deploy".to_string(),
        actor_id: "assistant".to_string(),
    };
    let result = store
        .write_entry(
            &scope,
            MemoryEntry {
                memory_id: "memory-fallback".to_string(),
                summary: "fallback should keep memory writes online".to_string(),
                tags: vec!["incident".to_string()],
                facts: vec!["provider=down".to_string()],
                source_event_key: "evt-fallback".to_string(),
                recency_weight_bps: 100,
                confidence_bps: 900,
            },
        )
        .expect("fallback write");

    assert_eq!(result.record.embedding_source, "hash-fnv1a");
    assert_eq!(result.record.embedding_model, None);
    assert_eq!(
        result.record.embedding_reason_code,
        "memory_embedding_provider_failed"
    );
    assert_eq!(result.record.embedding_vector.len(), 16);
}

