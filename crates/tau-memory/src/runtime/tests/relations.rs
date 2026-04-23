//! Relation-handling tests for the memory runtime.
//!
//! Extracted from `runtime/tests.rs` per the 2026-04-23 split audit
//! (Proposal 3, increment 5b). Covers spec_2592 (MemoryRelationType enum
//! roundtrip + normalize_relations enum-only acceptance),
//! unit_normalize_relations_validates_target_type_and_weight, and the
//! write/read integration tests for relations persisted in SQLite.

use super::super::{FileMemoryStore, MemoryType};
use crate::memory_contract::{MemoryEntry, MemoryScope};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn spec_2592_c01_memory_relation_enum_canonical_roundtrip() {
    let canonical = [
        "related_to",
        "updates",
        "contradicts",
        "caused_by",
        "result_of",
        "part_of",
    ];
    for label in canonical {
        let relation = super::super::MemoryRelationType::parse(label)
            .expect("canonical relation type must parse");
        assert_eq!(relation.as_str(), label);
        let encoded = serde_json::to_string(&relation).expect("serialize relation enum");
        assert_eq!(encoded, format!("\"{label}\""));
        let decoded: super::super::MemoryRelationType =
            serde_json::from_str(encoded.as_str()).expect("deserialize relation enum");
        assert_eq!(decoded, relation);
    }
    assert_eq!(
        super::super::MemoryRelationType::parse("depends_on"),
        Some(super::super::MemoryRelationType::CausedBy)
    );
    assert_eq!(
        super::super::MemoryRelationType::parse("relates_to"),
        Some(super::super::MemoryRelationType::RelatedTo)
    );
    assert_eq!(super::super::MemoryRelationType::parse(""), None);
    assert_eq!(super::super::MemoryRelationType::parse("unknown"), None);
}

#[test]
fn spec_2592_c02_normalize_relations_accepts_only_supported_relation_enum_values() {
    let known_memory_ids = std::collections::BTreeSet::from([String::from("target-memory")]);
    let valid = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: Some("updates".to_string()),
            weight: Some(0.75),
        }],
        &known_memory_ids,
    )
    .expect("valid relation normalization");
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].target_id, "target-memory");
    assert_eq!(valid[0].relation_type.as_str(), "updates");
    assert!((valid[0].weight - 0.75).abs() <= 0.000_001);
    assert!((valid[0].effective_weight - 0.75).abs() <= 0.000_001);

    let default_type = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: None,
            weight: None,
        }],
        &known_memory_ids,
    )
    .expect("default relation type and weight");
    assert_eq!(default_type[0].relation_type.as_str(), "related_to");
    assert!((default_type[0].weight - 1.0).abs() <= 0.000_001);
    assert!((default_type[0].effective_weight - 1.0).abs() <= 0.000_001);

    let invalid_legacy = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: Some("unknown_relation".to_string()),
            weight: Some(0.5),
        }],
        &known_memory_ids,
    )
    .expect_err("unsupported relation labels must fail closed");
    assert!(invalid_legacy
        .to_string()
        .contains("unsupported relation_type"));
}

#[test]
fn unit_normalize_relations_validates_target_type_and_weight() {
    let known_memory_ids = std::collections::BTreeSet::from([String::from("target-memory")]);
    let valid = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: Some("depends_on".to_string()),
            weight: Some(0.75),
        }],
        &known_memory_ids,
    )
    .expect("valid relation normalization");
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].target_id, "target-memory");
    assert_eq!(valid[0].relation_type, super::super::MemoryRelationType::CausedBy);
    assert!((valid[0].weight - 0.75).abs() <= 0.000_001);
    assert!((valid[0].effective_weight - 0.75).abs() <= 0.000_001);

    let default_type = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: None,
            weight: None,
        }],
        &known_memory_ids,
    )
    .expect("default relation type and weight");
    assert_eq!(
        default_type[0].relation_type,
        super::super::MemoryRelationType::RelatedTo
    );
    assert!((default_type[0].weight - 1.0).abs() <= 0.000_001);
    assert!((default_type[0].effective_weight - 1.0).abs() <= 0.000_001);

    let unknown_target = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "missing-target".to_string(),
            relation_type: Some("depends_on".to_string()),
            weight: Some(0.5),
        }],
        &known_memory_ids,
    )
    .expect_err("unknown target must fail");
    assert!(unknown_target
        .to_string()
        .contains("memory_invalid_relation"));

    let self_target_known = std::collections::BTreeSet::from([String::from("source-memory")]);
    let self_target = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "source-memory".to_string(),
            relation_type: Some("depends_on".to_string()),
            weight: Some(0.5),
        }],
        &self_target_known,
    )
    .expect_err("self target must fail");
    assert!(self_target.to_string().contains("must differ"));

    let invalid_type = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: Some("unknown".to_string()),
            weight: Some(0.5),
        }],
        &known_memory_ids,
    )
    .expect_err("invalid type must fail");
    assert!(invalid_type
        .to_string()
        .contains("unsupported relation_type"));

    let invalid_weight = super::super::normalize_relations(
        "source-memory",
        &[super::super::MemoryRelationInput {
            target_id: "target-memory".to_string(),
            relation_type: Some("depends_on".to_string()),
            weight: Some(1.5),
        }],
        &known_memory_ids,
    )
    .expect_err("invalid weight must fail");
    assert!(invalid_weight.to_string().contains("0.0..=1.0"));
}

#[test]
fn regression_write_entry_with_relations_created_flag_tracks_scope_membership() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope_a = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "ops".to_string(),
        actor_id: "assistant".to_string(),
    };
    let scope_b = MemoryScope {
        workspace_id: "workspace-a".to_string(),
        channel_id: "ops-secondary".to_string(),
        actor_id: "assistant".to_string(),
    };
    let entry = MemoryEntry {
        memory_id: "shared-memory".to_string(),
        summary: "shared summary".to_string(),
        tags: Vec::new(),
        facts: Vec::new(),
        source_event_key: "evt-shared".to_string(),
        recency_weight_bps: 0,
        confidence_bps: 1_000,
    };

    let first = store
        .write_entry_with_metadata_and_relations(
            &scope_a,
            entry.clone(),
            Some(MemoryType::Fact),
            Some(0.65),
            &[],
        )
        .expect("first write");
    assert!(first.created);

    let second_same_scope = store
        .write_entry_with_metadata_and_relations(
            &scope_a,
            entry.clone(),
            Some(MemoryType::Fact),
            Some(0.65),
            &[],
        )
        .expect("second write same scope");
    assert!(!second_same_scope.created);

    let third_other_scope = store
        .write_entry_with_metadata_and_relations(
            &scope_b,
            entry,
            Some(MemoryType::Fact),
            Some(0.65),
            &[],
        )
        .expect("third write other scope");
    assert!(third_other_scope.created);
}

#[test]
fn integration_read_entry_hydrates_relations_from_sqlite_relation_table() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let sqlite_path = store.storage_path().expect("sqlite path").to_path_buf();
    let connection =
        super::super::open_memory_sqlite_connection(&sqlite_path).expect("open sqlite memory store");
    super::super::initialize_memory_sqlite_schema(&connection).expect("initialize schema");

    let source_json = json!({
        "schema_version": 1,
        "updated_unix_ms": 100,
        "scope": {
            "workspace_id": "workspace-a",
            "channel_id": "ops",
            "actor_id": "assistant"
        },
        "entry": {
            "memory_id": "source-legacy",
            "summary": "legacy source entry",
            "tags": [],
            "facts": [],
            "source_event_key": "evt-source",
            "recency_weight_bps": 0,
            "confidence_bps": 1000
        },
        "memory_type": "observation",
        "importance": 0.3,
        "embedding_source": "hash-fnv1a",
        "embedding_model": null,
        "embedding_vector": [0.1, 0.2],
        "embedding_reason_code": "memory_embedding_hash_only"
    })
    .to_string();
    connection
        .execute(
            r#"
            INSERT INTO memory_records (memory_id, updated_unix_ms, record_json)
            VALUES (?1, ?2, ?3)
            "#,
            rusqlite::params!["source-legacy", 100_i64, source_json],
        )
        .expect("insert source legacy record");

    let target_json = json!({
        "schema_version": 1,
        "updated_unix_ms": 90,
        "scope": {
            "workspace_id": "workspace-a",
            "channel_id": "ops",
            "actor_id": "assistant"
        },
        "entry": {
            "memory_id": "target-legacy",
            "summary": "legacy target entry",
            "tags": [],
            "facts": [],
            "source_event_key": "evt-target",
            "recency_weight_bps": 0,
            "confidence_bps": 1000
        },
        "memory_type": "goal",
        "importance": 1.0,
        "embedding_source": "hash-fnv1a",
        "embedding_model": null,
        "embedding_vector": [0.1, 0.2],
        "embedding_reason_code": "memory_embedding_hash_only"
    })
    .to_string();
    connection
        .execute(
            r#"
            INSERT INTO memory_records (memory_id, updated_unix_ms, record_json)
            VALUES (?1, ?2, ?3)
            "#,
            rusqlite::params!["target-legacy", 90_i64, target_json],
        )
        .expect("insert target legacy record");

    connection
        .execute(
            r#"
            INSERT INTO memory_relations (
                source_memory_id,
                target_memory_id,
                relation_type,
                weight,
                effective_weight,
                updated_unix_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            rusqlite::params![
                "source-legacy",
                "target-legacy",
                "depends_on",
                0.7_f32,
                0.7_f32,
                100_i64
            ],
        )
        .expect("insert relation edge");

    let read = store
        .read_entry("source-legacy", None)
        .expect("read source")
        .expect("source exists");
    assert_eq!(read.relations.len(), 1);
    assert_eq!(read.relations[0].target_id, "target-legacy");
    assert_eq!(
        read.relations[0].relation_type,
        super::super::MemoryRelationType::CausedBy
    );
    assert!((read.relations[0].effective_weight - 0.7).abs() <= 0.000_001);
}

