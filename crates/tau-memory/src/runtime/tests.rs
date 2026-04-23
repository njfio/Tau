//! Runtime-level unit tests for `tau-memory`.
//!
//! Extracted from `runtime.rs` per the 2026-04-23 split audit (Proposal 3,
//! increment 4). Content unchanged from the previous inline `mod tests {}`
//! block; `super::` paths continue to resolve to the runtime module because
//! this file is still a direct child of `runtime`.

    use super::{
        embed_text_vector, importance_rank_multiplier, rank_text_candidates,
        rank_text_candidates_bm25, FileMemoryStore, MemoryIngestionLlmOptions,
        MemoryIngestionOptions, MemorySearchOptions, MemoryType, RankedTextCandidate,
        MEMORY_INGESTION_DEFAULT_CHUNK_LINE_COUNT, MEMORY_INGESTION_LLM_DEFAULT_TIMEOUT_MS,
    };
    use crate::memory_contract::{MemoryEntry, MemoryScope};
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

    mod search;
    mod store;

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
