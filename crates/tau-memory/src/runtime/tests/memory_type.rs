//! MemoryType / importance-profile tests for the memory runtime.
//!
//! Extracted from `runtime/tests.rs` per the 2026-04-23 split audit
//! (Proposal 3, increment 5c). Covers MemoryType parse/display
//! roundtrip, default importance profile behavior, importance-profile
//! mutation + validation, and spec_2589_c02 (configured-type default
//! importance on FileMemoryStore).

use super::super::{FileMemoryStore, MemoryType, MemoryTypeImportanceProfile, RuntimeMemoryRecord};
use super::lifecycle::{lifecycle_entry, lifecycle_scope};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn unit_memory_type_parse_and_display_roundtrip() {
    let cases = [
        (MemoryType::Identity, "identity"),
        (MemoryType::Goal, "goal"),
        (MemoryType::Decision, "decision"),
        (MemoryType::Todo, "todo"),
        (MemoryType::Preference, "preference"),
        (MemoryType::Fact, "fact"),
        (MemoryType::Event, "event"),
        (MemoryType::Observation, "observation"),
    ];
    for (memory_type, label) in cases {
        let uppercase = label.to_ascii_uppercase();
        let padded = format!(" {label} ");
        assert_eq!(memory_type.as_str(), label);
        assert_eq!(MemoryType::parse(label), Some(memory_type));
        assert_eq!(MemoryType::parse(uppercase.as_str()), Some(memory_type));
        assert_eq!(MemoryType::parse(padded.as_str()), Some(memory_type));
    }
    assert_eq!(MemoryType::parse("unknown"), None);
}

#[test]
fn unit_memory_type_default_importance_profile_and_record_defaults() {
    let expectations = [
        (MemoryType::Identity, 1.0f32),
        (MemoryType::Goal, 0.9f32),
        (MemoryType::Decision, 0.85f32),
        (MemoryType::Todo, 0.8f32),
        (MemoryType::Preference, 0.7f32),
        (MemoryType::Fact, 0.65f32),
        (MemoryType::Event, 0.55f32),
        (MemoryType::Observation, 0.3f32),
    ];
    for (memory_type, expected_importance) in expectations {
        assert!(
            (memory_type.default_importance() - expected_importance).abs() <= 0.000_001,
            "default importance mismatch for {}",
            memory_type.as_str()
        );
    }
    assert_eq!(MemoryType::default(), MemoryType::Observation);

    let decoded: RuntimeMemoryRecord = serde_json::from_value(json!({
        "schema_version": 1,
        "updated_unix_ms": 123,
        "scope": {
            "workspace_id": "workspace",
            "channel_id": "channel",
            "actor_id": "assistant"
        },
        "entry": {
            "memory_id": "memory-default",
            "summary": "default metadata",
            "tags": [],
            "facts": [],
            "source_event_key": "evt-default",
            "recency_weight_bps": 0,
            "confidence_bps": 1000
        }
    }))
    .expect("deserialize runtime record with defaults");
    assert_eq!(decoded.memory_type, MemoryType::Observation);
    assert!((decoded.importance - 0.3).abs() <= 0.000_001);
    assert!(decoded.relations.is_empty());
    assert_eq!(decoded.last_accessed_at_unix_ms, 0);
    assert_eq!(decoded.access_count, 0);
    assert!(!decoded.forgotten);
}

#[test]
fn unit_memory_type_importance_profile_set_importance_updates_selected_type() {
    let mut profile = MemoryTypeImportanceProfile::default();
    let initial_goal = profile.goal;
    profile.set_importance(MemoryType::Identity, 0.44);
    profile.set_importance(MemoryType::Observation, 0.11);

    assert!((profile.identity - 0.44).abs() <= 0.000_001);
    assert!((profile.observation - 0.11).abs() <= 0.000_001);
    assert!((profile.goal - initial_goal).abs() <= 0.000_001);
}

#[test]
fn unit_memory_type_importance_profile_validate_rejects_invalid_values() {
    let non_finite = MemoryTypeImportanceProfile {
        identity: f32::INFINITY,
        ..Default::default()
    };
    let non_finite_error = non_finite
        .validate()
        .expect_err("non-finite defaults must fail validation");
    assert!(non_finite_error.to_string().contains("identity"));

    let out_of_range = MemoryTypeImportanceProfile {
        goal: 1.5,
        ..Default::default()
    };
    let out_of_range_error = out_of_range
        .validate()
        .expect_err("out-of-range defaults must fail validation");
    assert!(out_of_range_error.to_string().contains("goal"));

    let negative = MemoryTypeImportanceProfile {
        todo: -0.01,
        ..Default::default()
    };
    let negative_error = negative
        .validate()
        .expect_err("negative defaults must fail validation");
    assert!(negative_error.to_string().contains("todo"));
}

#[test]
fn spec_2589_c02_file_memory_store_applies_configured_type_default_importance() {
    let temp = tempdir().expect("tempdir");
    let profile = MemoryTypeImportanceProfile {
        identity: 0.42,
        observation: 0.18,
        ..Default::default()
    };

    let store = FileMemoryStore::new_with_embedding_provider_and_importance_profile(
        temp.path(),
        None,
        Some(profile),
    );
    let resolved_profile = store.default_importance_profile();
    assert!((resolved_profile.identity - 0.42).abs() <= 0.000_001);
    assert!((resolved_profile.observation - 0.18).abs() <= 0.000_001);
    let scope = lifecycle_scope();

    let identity = store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry(
                "memory-configured-identity",
                "identity from configured profile",
            ),
            Some(MemoryType::Identity),
            None,
        )
        .expect("write identity with configured fallback");
    assert!((identity.record.importance - 0.42).abs() <= 0.000_001);

    let observation = store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry(
                "memory-configured-observation",
                "observation from configured profile",
            ),
            None,
            None,
        )
        .expect("write observation with configured fallback");
    assert_eq!(observation.record.memory_type, MemoryType::Observation);
    assert!((observation.record.importance - 0.18).abs() <= 0.000_001);
}

