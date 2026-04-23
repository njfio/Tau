//! Runtime-level unit tests for `tau-memory`.
//!
//! Extracted from `runtime.rs` per the 2026-04-23 split audit (Proposal 3,
//! increment 4). Content unchanged from the previous inline `mod tests {}`
//! block; `super::` paths continue to resolve to the runtime module because
//! this file is still a direct child of `runtime`.

    use super::{
        embed_text_vector, importance_rank_multiplier, rank_text_candidates,
        rank_text_candidates_bm25, FileMemoryStore, MemoryEmbeddingProviderConfig,
        MemoryIngestionLlmOptions, MemoryIngestionOptions, MemoryScopeFilter,
        MemorySearchOptions, MemoryType, RankedTextCandidate,
        MEMORY_INGESTION_DEFAULT_CHUNK_LINE_COUNT, MEMORY_INGESTION_LLM_DEFAULT_TIMEOUT_MS,
    };
    use crate::memory_contract::{MemoryEntry, MemoryScope};
    use httpmock::{Method::POST, MockServer};
    use serde_json::json;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    pub(super) struct ScopedEnvVar {
        pub(super) key: &'static str,
        pub(super) previous: Option<String>,
    }

    impl ScopedEnvVar {
        pub(super) fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            match self.previous.as_deref() {
                Some(previous) => std::env::set_var(self.key, previous),
                None => std::env::remove_var(self.key),
            }
        }
    }

    pub(super) fn memory_backend_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn regression_spec_3412_sqlite_i64_from_u64_rejects_values_outside_sqlite_integer_range() {
        let error = super::sqlite_i64_from_u64(u64::MAX, "updated_unix_ms")
            .expect_err("u64::MAX should fail sqlite integer conversion");
        assert!(error.to_string().contains("updated_unix_ms value"));
        assert!(error.to_string().contains("SQLite INTEGER range"));
    }

    #[test]
    fn regression_spec_3412_sqlite_u64_from_i64_rejects_negative_sqlite_integer_values() {
        let error = super::sqlite_u64_from_i64(-1, "memory_records_count")
            .expect_err("negative sqlite integer should fail u64 conversion");
        assert!(error.to_string().contains("memory_records_count value -1"));
        assert!(error.to_string().contains("non-negative SQLite INTEGER"));
    }
    mod lifecycle;

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

    mod memory_type;

    #[test]
    fn unit_memory_search_options_serde_default_sets_graph_signal_weight() {
        let decoded: MemorySearchOptions = serde_json::from_value(json!({
            "scope": {
                "workspace_id": null,
                "channel_id": null,
                "actor_id": null
            },
            "limit": 5,
            "embedding_dimensions": 128,
            "min_similarity": 0.55,
            "enable_hybrid_retrieval": false,
            "bm25_k1": 1.2,
            "bm25_b": 0.75,
            "bm25_min_score": 0.0,
            "rrf_k": 60,
            "rrf_vector_weight": 1.0,
            "rrf_lexical_weight": 1.0,
            "enable_embedding_migration": true,
            "benchmark_against_hash": false,
            "benchmark_against_vector_only": false
        }))
        .expect("deserialize search options with graph default");
        assert!((decoded.graph_signal_weight - 0.25).abs() <= 0.000_001);
    }

    #[test]
    fn regression_spec_2492_c05_ingestion_options_serde_defaults_are_contract_stable() {
        let decoded: MemoryIngestionOptions =
            serde_json::from_value(json!({})).expect("deserialize ingestion defaults");
        assert_eq!(
            decoded.chunk_line_count,
            MEMORY_INGESTION_DEFAULT_CHUNK_LINE_COUNT
        );
        assert!(decoded.delete_source_on_success);
        assert_eq!(decoded.scope.workspace_id, "default-workspace");
        assert_eq!(decoded.scope.channel_id, "default-channel");
        assert_eq!(decoded.scope.actor_id, "default-actor");
    }

    #[test]
    fn regression_spec_2503_c10_llm_options_default_timeout_is_contract_stable() {
        let decoded: MemoryIngestionLlmOptions = serde_json::from_value(json!({
            "provider": "openai-compatible",
            "model": "gpt-5.2",
            "api_base": "https://example.invalid",
            "api_key": "test"
        }))
        .expect("deserialize llm ingestion options");
        assert_eq!(
            decoded.timeout_ms, MEMORY_INGESTION_LLM_DEFAULT_TIMEOUT_MS,
            "llm options timeout default drifted"
        );
    }

    mod relations;

    #[test]
    fn unit_importance_rank_multiplier_clamps_to_expected_range() {
        assert!((importance_rank_multiplier(-1.0) - 1.0).abs() <= 0.000_001);
        assert!((importance_rank_multiplier(0.0) - 1.0).abs() <= 0.000_001);
        assert!((importance_rank_multiplier(0.5) - 1.5).abs() <= 0.000_001);
        assert!((importance_rank_multiplier(1.0) - 2.0).abs() <= 0.000_001);
        assert!((importance_rank_multiplier(3.0) - 2.0).abs() <= 0.000_001);
    }

    #[test]
    fn regression_write_entry_with_metadata_rejects_invalid_importance_range() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "channel-1".to_string(),
            actor_id: "assistant".to_string(),
        };
        let base_entry = MemoryEntry {
            memory_id: "memory-invalid-importance".to_string(),
            summary: "importance must remain bounded".to_string(),
            tags: vec!["validation".to_string()],
            facts: vec!["range=0..1".to_string()],
            source_event_key: "evt-invalid".to_string(),
            recency_weight_bps: 0,
            confidence_bps: 1_000,
        };

        let below = store.write_entry_with_metadata(
            &scope,
            base_entry.clone(),
            Some(MemoryType::Goal),
            Some(-0.01),
        );
        assert!(below.is_err());

        let above = store.write_entry_with_metadata(
            &scope,
            base_entry.clone(),
            Some(MemoryType::Goal),
            Some(1.01),
        );
        assert!(above.is_err());

        let nan = store.write_entry_with_metadata(
            &scope,
            base_entry.clone(),
            Some(MemoryType::Goal),
            Some(f32::NAN),
        );
        assert!(nan.is_err());

        let valid = store
            .write_entry_with_metadata(
                &scope,
                MemoryEntry {
                    memory_id: "memory-valid-importance".to_string(),
                    ..base_entry
                },
                Some(MemoryType::Goal),
                Some(0.95),
            )
            .expect("valid importance should write successfully");
        assert_eq!(valid.record.memory_type, MemoryType::Goal);
        assert!((valid.record.importance - 0.95).abs() <= 0.000_001);
    }

    #[test]
    fn integration_memory_search_importance_multiplier_prioritizes_high_importance_match() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "deploy".to_string(),
            actor_id: "assistant".to_string(),
        };

        let shared_summary = "release smoke checklist".to_string();
        let shared_tags = vec!["release".to_string()];
        let shared_facts = vec!["run smoke tests".to_string()];

        store
            .write_entry_with_metadata(
                &scope,
                MemoryEntry {
                    memory_id: "a-low".to_string(),
                    summary: shared_summary.clone(),
                    tags: shared_tags.clone(),
                    facts: shared_facts.clone(),
                    source_event_key: "evt-low".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 1_000,
                },
                Some(MemoryType::Observation),
                Some(0.0),
            )
            .expect("write low importance");
        store
            .write_entry_with_metadata(
                &scope,
                MemoryEntry {
                    memory_id: "z-high".to_string(),
                    summary: shared_summary,
                    tags: shared_tags,
                    facts: shared_facts,
                    source_event_key: "evt-high".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 1_000,
                },
                Some(MemoryType::Goal),
                Some(1.0),
            )
            .expect("write high importance");

        let result = store
            .search(
                "release smoke checklist",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 64,
                    min_similarity: 0.0,
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.0,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: false,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("search with importance ranking");

        assert_eq!(result.returned, 2);
        assert_eq!(result.matches[0].memory_id, "z-high");
        assert!(result.matches[0].score > result.matches[1].score);
        let low = result
            .matches
            .iter()
            .find(|item| item.memory_id == "a-low")
            .expect("low memory in ranked matches")
            .score;
        let high = result
            .matches
            .iter()
            .find(|item| item.memory_id == "z-high")
            .expect("high memory in ranked matches")
            .score;
        assert!(low > 0.0);
        let ratio = high / low;
        assert!(
            (ratio - 2.0).abs() <= 0.05,
            "importance multiplier ratio drifted: {ratio}"
        );
    }

    #[test]
    fn integration_migrate_records_to_provider_embeddings_reports_count_and_preserves_metadata() {
        let server = MockServer::start();
        let embeddings = server.mock(|when, then| {
            when.method(POST).path("/embeddings");
            then.status(200).json_body_obj(&json!({
                "data": [
                    { "embedding": [0.9, 0.1, 0.0, 0.0] },
                    { "embedding": [0.8, 0.2, 0.0, 0.0] }
                ]
            }));
        });

        let temp = tempdir().expect("tempdir");
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "ops".to_string(),
            actor_id: "assistant".to_string(),
        };
        let seed_store = FileMemoryStore::new(temp.path());
        seed_store
            .write_entry_with_metadata(
                &scope,
                MemoryEntry {
                    memory_id: "memory-migrate-a".to_string(),
                    summary: "provider migration candidate a".to_string(),
                    tags: vec!["migration".to_string()],
                    facts: vec!["priority=high".to_string()],
                    source_event_key: "evt-migrate-a".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 1_000,
                },
                Some(MemoryType::Goal),
                Some(0.9),
            )
            .expect("write migration candidate a");
        seed_store
            .write_entry_with_metadata(
                &scope,
                MemoryEntry {
                    memory_id: "memory-migrate-b".to_string(),
                    summary: "provider migration candidate b".to_string(),
                    tags: vec!["migration".to_string()],
                    facts: vec!["priority=medium".to_string()],
                    source_event_key: "evt-migrate-b".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 1_000,
                },
                Some(MemoryType::Fact),
                Some(0.65),
            )
            .expect("write migration candidate b");

        let records = seed_store
            .list_latest_records(None, usize::MAX)
            .expect("list seeded records");
        assert_eq!(records.len(), 2);

        let provider_store = FileMemoryStore::new_with_embedding_provider(
            temp.path(),
            Some(MemoryEmbeddingProviderConfig {
                provider: "openai-compatible".to_string(),
                model: "text-embedding-3-small".to_string(),
                api_base: server.url(""),
                api_key: "test-key".to_string(),
                dimensions: 4,
                timeout_ms: 5_000,
            }),
        );
        let migrated = provider_store
            .migrate_records_to_provider_embeddings(&records)
            .expect("migrate records to provider embeddings");
        assert_eq!(migrated, 2);
        embeddings.assert();

        let migrated_a = provider_store
            .read_entry("memory-migrate-a", None)
            .expect("read migrated a")
            .expect("migrated a exists");
        let migrated_b = provider_store
            .read_entry("memory-migrate-b", None)
            .expect("read migrated b")
            .expect("migrated b exists");
        assert_eq!(migrated_a.embedding_source, "provider-openai-compatible");
        assert_eq!(migrated_b.embedding_source, "provider-openai-compatible");
        assert_eq!(migrated_a.memory_type, MemoryType::Goal);
        assert_eq!(migrated_b.memory_type, MemoryType::Fact);
        assert!((migrated_a.importance - 0.9).abs() <= 0.000_001);
        assert!((migrated_b.importance - 0.65).abs() <= 0.000_001);
    }

    mod store;

    #[test]
    fn integration_memory_search_migrates_hash_records_to_provider_embeddings() {
        let temp = tempdir().expect("tempdir");
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "deploy".to_string(),
            actor_id: "assistant".to_string(),
        };
        let hash_store = FileMemoryStore::new(temp.path());
        hash_store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-1".to_string(),
                    summary: "release workflow validation".to_string(),
                    tags: vec!["release".to_string()],
                    facts: vec!["check smoke tests".to_string()],
                    source_event_key: "evt-1".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write first hash record");
        hash_store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-2".to_string(),
                    summary: "release freeze checklist".to_string(),
                    tags: vec!["freeze".to_string()],
                    facts: vec!["validate rollback".to_string()],
                    source_event_key: "evt-2".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write second hash record");

        let server = MockServer::start();
        let migration_call = server.mock(|when, then| {
            when.method(POST)
                .path("/embeddings")
                .body_includes("release workflow validation");
            then.status(200).json_body_obj(&serde_json::json!({
                "data": [
                    { "embedding": [0.9, 0.0, 0.1, 0.0] },
                    { "embedding": [0.8, 0.0, 0.2, 0.0] }
                ]
            }));
        });
        let query_call = server.mock(|when, then| {
            when.method(POST)
                .path("/embeddings")
                .body_includes("release workflow");
            then.status(200).json_body_obj(&serde_json::json!({
                "data": [
                    { "embedding": [0.95, 0.0, 0.05, 0.0] }
                ]
            }));
        });

        let provider_store = FileMemoryStore::new_with_embedding_provider(
            temp.path(),
            Some(MemoryEmbeddingProviderConfig {
                provider: "openai-compatible".to_string(),
                model: "text-embedding-3-small".to_string(),
                api_base: server.url(""),
                api_key: "test-key".to_string(),
                dimensions: 4,
                timeout_ms: 5_000,
            }),
        );
        let result = provider_store
            .search(
                "release workflow",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 4,
                    min_similarity: 0.0,
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.0,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: true,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("search with migration");

        migration_call.assert();
        query_call.assert();
        assert_eq!(result.migrated_entries, 2);
        assert_eq!(result.embedding_backend, "provider-openai-compatible");
        assert_eq!(
            result.embedding_reason_code,
            "memory_embedding_provider_success"
        );
        assert!(result.returned >= 1);

        let migrated_first = provider_store
            .read_entry("memory-1", None)
            .expect("read migrated first")
            .expect("first exists");
        let migrated_second = provider_store
            .read_entry("memory-2", None)
            .expect("read migrated second")
            .expect("second exists");
        assert_eq!(
            migrated_first.embedding_source,
            "provider-openai-compatible"
        );
        assert_eq!(
            migrated_second.embedding_source,
            "provider-openai-compatible"
        );
        assert_eq!(
            migrated_first.embedding_reason_code,
            "memory_embedding_provider_success"
        );
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
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.0,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: true,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("search");
        assert_eq!(result.returned, 1);
        assert_eq!(result.matches[0].memory_id, "memory-release");
        assert_eq!(result.matches[0].scope.workspace_id, "workspace-a");
    }

    #[test]
    fn regression_memory_search_reports_baseline_overlap_when_benchmark_enabled() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "deploy".to_string(),
            actor_id: "assistant".to_string(),
        };
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-release".to_string(),
                    summary: "release smoke checklist".to_string(),
                    tags: vec!["release".to_string()],
                    facts: vec!["smoke".to_string()],
                    source_event_key: "evt-1".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write release memory");
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-unrelated".to_string(),
                    summary: "office lunch planning".to_string(),
                    tags: vec!["social".to_string()],
                    facts: vec!["pizza".to_string()],
                    source_event_key: "evt-2".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write unrelated memory");

        let benchmarked = store
            .search(
                "release smoke",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 64,
                    min_similarity: 0.0,
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.0,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: false,
                    benchmark_against_hash: true,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("benchmarked search");
        let regular = store
            .search(
                "release smoke",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 64,
                    min_similarity: 0.0,
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.0,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: false,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("regular search");

        assert!(benchmarked.baseline_hash_overlap.is_some());
        assert_eq!(regular.baseline_hash_overlap, None);
    }

    #[test]
    fn integration_memory_search_hybrid_returns_lexical_match_when_vector_filter_excludes() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "ops".to_string(),
            actor_id: "assistant".to_string(),
        };
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-hybrid".to_string(),
                    summary: "kubernetes incident playbook for oncall".to_string(),
                    tags: vec!["kubernetes".to_string()],
                    facts: vec!["incident escalation".to_string()],
                    source_event_key: "evt-hybrid".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write hybrid memory");
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-other".to_string(),
                    summary: "office kitchen cleanup schedule".to_string(),
                    tags: vec!["office".to_string()],
                    facts: vec!["cleanup rota".to_string()],
                    source_event_key: "evt-other".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write other memory");

        let vector_only = store
            .search(
                "kubernetes incident",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 64,
                    min_similarity: 1.1,
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.1,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: false,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("vector-only search");
        let hybrid = store
            .search(
                "kubernetes incident",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 64,
                    min_similarity: 1.1,
                    enable_hybrid_retrieval: true,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.1,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: false,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: true,
                },
            )
            .expect("hybrid search");

        assert_eq!(vector_only.returned, 0);
        assert_eq!(hybrid.returned, 1);
        assert_eq!(hybrid.matches[0].memory_id, "memory-hybrid");
        assert_eq!(hybrid.retrieval_backend, "hybrid-bm25-rrf");
        assert_eq!(
            hybrid.retrieval_reason_code,
            "memory_retrieval_hybrid_enabled"
        );
        assert!(hybrid.matches[0]
            .lexical_score
            .is_some_and(|score| score > 0.0));
        assert!(hybrid.matches[0].vector_score.is_none());
        assert!(hybrid.baseline_vector_overlap.is_some());
    }

    #[test]
    fn regression_memory_search_vector_only_matches_hash_baseline_order() {
        let temp = tempdir().expect("tempdir");
        let store = FileMemoryStore::new(temp.path());
        let scope = MemoryScope {
            workspace_id: "workspace-a".to_string(),
            channel_id: "deploy".to_string(),
            actor_id: "assistant".to_string(),
        };

        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-a".to_string(),
                    summary: "release checklist smoke tests".to_string(),
                    tags: vec!["release".to_string()],
                    facts: vec!["smoke".to_string()],
                    source_event_key: "evt-a".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write memory a");
        store
            .write_entry(
                &scope,
                MemoryEntry {
                    memory_id: "memory-b".to_string(),
                    summary: "deployment rollback strategy".to_string(),
                    tags: vec!["rollback".to_string()],
                    facts: vec!["rollback drill".to_string()],
                    source_event_key: "evt-b".to_string(),
                    recency_weight_bps: 0,
                    confidence_bps: 0,
                },
            )
            .expect("write memory b");

        let result = store
            .search(
                "release smoke",
                &MemorySearchOptions {
                    scope: MemoryScopeFilter::default(),
                    limit: 5,
                    embedding_dimensions: 64,
                    min_similarity: 0.0,
                    enable_hybrid_retrieval: false,
                    bm25_k1: 1.2,
                    bm25_b: 0.75,
                    bm25_min_score: 0.0,
                    rrf_k: 60,
                    rrf_vector_weight: 1.0,
                    rrf_lexical_weight: 1.0,
                    graph_signal_weight: 0.25,
                    enable_embedding_migration: false,
                    benchmark_against_hash: false,
                    benchmark_against_vector_only: false,
                },
            )
            .expect("vector-only search");
        let records = store
            .list_latest_records(None, usize::MAX)
            .expect("list latest records");
        let baseline = rank_text_candidates(
            "release smoke",
            records
                .iter()
                .map(|record| RankedTextCandidate {
                    key: record.entry.memory_id.clone(),
                    text: format!(
                        "{}\n{}\n{}",
                        record.entry.summary,
                        record.entry.tags.join(" "),
                        record.entry.facts.join(" ")
                    ),
                })
                .collect::<Vec<_>>(),
            5,
            64,
            0.0,
        );
        let result_ids = result
            .matches
            .iter()
            .map(|item| item.memory_id.as_str())
            .collect::<Vec<_>>();
        let baseline_ids = baseline
            .iter()
            .map(|item| item.key.as_str())
            .collect::<Vec<_>>();

        assert_eq!(result_ids, baseline_ids);
        assert_eq!(result.retrieval_backend, "vector-only");
        assert_eq!(result.retrieval_reason_code, "memory_retrieval_vector_only");
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

    #[test]
    fn unit_rank_text_candidates_bm25_prefers_exact_lexical_overlap() {
        let ranked = rank_text_candidates_bm25(
            "tokio runtime",
            vec![
                RankedTextCandidate {
                    key: "match".to_string(),
                    text: "tokio runtime troubleshooting checklist".to_string(),
                },
                RankedTextCandidate {
                    key: "other".to_string(),
                    text: "garden watering schedule".to_string(),
                },
            ],
            5,
            1.2,
            0.75,
            0.001,
        );
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].key, "match");
        assert!(ranked[0].score > 0.0);
    }
