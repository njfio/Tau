//! Lifecycle-maintenance tests for the memory runtime.
//!
//! Extracted from `runtime/tests.rs` per the 2026-04-23 split audit
//! (Proposal 3, increment 5a). Covers spec_2455 (decay / prune / orphan
//! cleanup), spec_2460 (near-duplicate cleanup), and their regression
//! tests.

use super::super::{
    FileMemoryStore, MemoryLifecycleMaintenancePolicy, MemoryLifecycleMaintenanceResult,
    MemoryRelationInput, MemorySearchOptions, MemoryType,
};
use crate::memory_contract::{MemoryEntry, MemoryScope};
use tempfile::tempdir;


pub(super) fn lifecycle_scope() -> MemoryScope {
    MemoryScope {
        workspace_id: "workspace-lifecycle".to_string(),
        channel_id: "channel-lifecycle".to_string(),
        actor_id: "assistant".to_string(),
    }
}

pub(super) fn lifecycle_entry(memory_id: &str, summary: &str) -> MemoryEntry {
    MemoryEntry {
        memory_id: memory_id.to_string(),
        summary: summary.to_string(),
        tags: vec!["lifecycle".to_string()],
        facts: vec!["phase=2".to_string()],
        source_event_key: format!("evt-{memory_id}"),
        recency_weight_bps: 0,
        confidence_bps: 1_000,
    }
}

fn append_lifecycle_snapshot(
    store: &FileMemoryStore,
    memory_id: &str,
    updated_unix_ms: u64,
    last_accessed_at_unix_ms: u64,
    importance: f32,
) {
    let mut record = store
        .list_latest_records(None, usize::MAX)
        .expect("list latest lifecycle records")
        .into_iter()
        .find(|candidate| candidate.entry.memory_id == memory_id)
        .expect("record exists for lifecycle snapshot");
    record.updated_unix_ms = updated_unix_ms;
    record.last_accessed_at_unix_ms = last_accessed_at_unix_ms;
    record.importance = importance;
    store
        .append_record_backend(&record)
        .expect("append lifecycle snapshot");
}

#[test]
fn spec_2455_c01_lifecycle_maintenance_policy_defaults_and_empty_result_are_deterministic() {
    let policy = MemoryLifecycleMaintenancePolicy::default();
    assert_eq!(
        policy.stale_after_unix_ms,
        7_u64 * 24 * 60 * 60 * 1_000,
        "default stale threshold should be seven days"
    );
    assert!((policy.decay_rate - 0.9).abs() <= 0.000_001);
    assert!((policy.prune_importance_floor - 0.1).abs() <= 0.000_001);
    assert!(policy.orphan_cleanup_enabled);
    assert!((policy.orphan_importance_floor - 0.2).abs() <= 0.000_001);
    assert!(!policy.duplicate_cleanup_enabled);
    assert!((policy.duplicate_similarity_threshold - 0.97).abs() <= 0.000_001);

    let zero = MemoryLifecycleMaintenanceResult::default();
    assert_eq!(zero.scanned_records, 0);
    assert_eq!(zero.decayed_records, 0);
    assert_eq!(zero.pruned_records, 0);
    assert_eq!(zero.orphan_forgotten_records, 0);
    assert_eq!(zero.duplicate_forgotten_records, 0);
    assert_eq!(zero.identity_exempt_records, 0);
    assert_eq!(zero.updated_records, 0);
    assert_eq!(zero.unchanged_records, 0);

    let store = FileMemoryStore::new(tempdir().expect("tempdir").path());
    let run = store
        .run_lifecycle_maintenance(&policy, 10_000)
        .expect("run lifecycle maintenance");
    assert_eq!(run.scanned_records, 0);
    assert_eq!(run.updated_records, 0);
}

#[test]
fn spec_2455_c02_stale_non_identity_records_decay_while_identity_is_exempt() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-stale-observation", "stale observation"),
            Some(MemoryType::Observation),
            Some(0.6),
        )
        .expect("write stale observation");
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-stale-identity", "stale identity"),
            Some(MemoryType::Identity),
            Some(0.2),
        )
        .expect("write stale identity");

    append_lifecycle_snapshot(&store, "memory-stale-observation", 1_000, 1_000, 0.6);
    append_lifecycle_snapshot(&store, "memory-stale-identity", 1_000, 1_000, 0.2);

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: 1_000,
                decay_rate: 0.5,
                prune_importance_floor: 0.05,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: false,
                duplicate_similarity_threshold: 0.97,
            },
            10_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(run.scanned_records, 2);
    assert_eq!(run.decayed_records, 1);
    assert_eq!(run.identity_exempt_records, 1);
    assert_eq!(run.pruned_records, 0);

    let latest = store
        .list_latest_records(None, usize::MAX)
        .expect("list post-maintenance");
    let observation = latest
        .iter()
        .find(|record| record.entry.memory_id == "memory-stale-observation")
        .expect("observation record present");
    assert!((observation.importance - 0.3).abs() <= 0.000_001);
    let identity = latest
        .iter()
        .find(|record| record.entry.memory_id == "memory-stale-identity")
        .expect("identity record present");
    assert!((identity.importance - 0.2).abs() <= 0.000_001);
    assert!(!identity.forgotten);
}

#[test]
fn spec_2455_c03_prune_floor_marks_low_importance_records_as_forgotten() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-prune-low", "low importance"),
            Some(MemoryType::Observation),
            Some(0.05),
        )
        .expect("write low importance memory");
    append_lifecycle_snapshot(&store, "memory-prune-low", 9_500, 9_500, 0.05);

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: 60_000,
                decay_rate: 1.0,
                prune_importance_floor: 0.1,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: false,
                duplicate_similarity_threshold: 0.97,
            },
            10_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(run.pruned_records, 1);

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list latest records after prune");
    assert!(
        listed
            .iter()
            .all(|record| record.entry.memory_id != "memory-prune-low"),
        "forgotten memory must be excluded from default list"
    );
    let read = store
        .read_entry("memory-prune-low", None)
        .expect("read after prune");
    assert!(
        read.is_none(),
        "forgotten memory must be excluded from default read"
    );
    let search = store
        .search("low importance", &MemorySearchOptions::default())
        .expect("search after prune");
    assert!(
        search
            .matches
            .iter()
            .all(|record| record.memory_id != "memory-prune-low"),
        "forgotten memory must be excluded from default search"
    );
}

#[test]
fn regression_2455_prune_floor_boundary_keeps_equal_importance_active() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-prune-boundary", "boundary importance"),
            Some(MemoryType::Observation),
            Some(0.1),
        )
        .expect("write boundary importance memory");

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: u64::MAX,
                decay_rate: 1.0,
                prune_importance_floor: 0.1,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: false,
                duplicate_similarity_threshold: 0.97,
            },
            10_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(run.pruned_records, 0);
    assert_eq!(run.updated_records, 0);
    assert_eq!(run.unchanged_records, 1);

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list latest records after boundary prune");
    assert!(
        listed
            .iter()
            .any(|record| record.entry.memory_id == "memory-prune-boundary"),
        "importance equal to prune floor must remain active"
    );
}

#[test]
fn spec_2455_c04_orphan_cleanup_forgets_low_importance_orphans_only() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();

    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-linked-target", "linked target"),
            Some(MemoryType::Goal),
            Some(0.9),
        )
        .expect("write linked target");
    store
        .write_entry_with_metadata_and_relations(
            &scope,
            lifecycle_entry("memory-linked-low", "linked low importance"),
            Some(MemoryType::Observation),
            Some(0.15),
            &[MemoryRelationInput {
                target_id: "memory-linked-target".to_string(),
                relation_type: Some("depends_on".to_string()),
                weight: Some(1.0),
            }],
        )
        .expect("write linked low-importance record");
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-orphan-low", "orphan low importance"),
            Some(MemoryType::Observation),
            Some(0.15),
        )
        .expect("write orphan low-importance record");

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: u64::MAX,
                decay_rate: 1.0,
                prune_importance_floor: 0.1,
                orphan_cleanup_enabled: true,
                orphan_importance_floor: 0.2,
                duplicate_cleanup_enabled: false,
                duplicate_similarity_threshold: 0.97,
            },
            10_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(run.orphan_forgotten_records, 1);
    assert_eq!(run.pruned_records, 0);

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list post orphan cleanup");
    assert!(
        listed
            .iter()
            .any(|record| record.entry.memory_id == "memory-linked-low"),
        "edge-connected low-importance memory should remain active"
    );
    assert!(
        listed
            .iter()
            .all(|record| record.entry.memory_id != "memory-orphan-low"),
        "orphan low-importance memory should be forgotten"
    );
}

#[test]
fn spec_2455_c05_identity_records_are_exempt_from_decay_prune_and_orphan_cleanup() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-identity-critical", "identity memory"),
            Some(MemoryType::Identity),
            Some(0.01),
        )
        .expect("write identity memory");
    append_lifecycle_snapshot(&store, "memory-identity-critical", 1_000, 1_000, 0.01);

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: 1_000,
                decay_rate: 0.5,
                prune_importance_floor: 0.1,
                orphan_cleanup_enabled: true,
                orphan_importance_floor: 0.2,
                duplicate_cleanup_enabled: false,
                duplicate_similarity_threshold: 0.97,
            },
            10_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(run.identity_exempt_records, 1);
    assert_eq!(run.decayed_records, 0);
    assert_eq!(run.pruned_records, 0);
    assert_eq!(run.orphan_forgotten_records, 0);

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list post maintenance");
    let identity = listed
        .iter()
        .find(|record| record.entry.memory_id == "memory-identity-critical")
        .expect("identity record remains");
    assert!((identity.importance - 0.01).abs() <= 0.000_001);
    assert!(!identity.forgotten);
}

#[test]
fn unit_lifecycle_maintenance_rejects_invalid_policy_values() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());

    let invalid_decay = store.run_lifecycle_maintenance(
        &MemoryLifecycleMaintenancePolicy {
            stale_after_unix_ms: 1_000,
            decay_rate: 1.5,
            prune_importance_floor: 0.1,
            orphan_cleanup_enabled: true,
            orphan_importance_floor: 0.2,
            duplicate_cleanup_enabled: true,
            duplicate_similarity_threshold: 0.95,
        },
        10_000,
    );
    assert!(invalid_decay.is_err(), "out-of-range decay_rate must fail");

    let invalid_prune_floor = store.run_lifecycle_maintenance(
        &MemoryLifecycleMaintenancePolicy {
            stale_after_unix_ms: 1_000,
            decay_rate: 0.9,
            prune_importance_floor: -0.1,
            orphan_cleanup_enabled: true,
            orphan_importance_floor: 0.2,
            duplicate_cleanup_enabled: true,
            duplicate_similarity_threshold: 0.95,
        },
        10_000,
    );
    assert!(
        invalid_prune_floor.is_err(),
        "negative prune_importance_floor must fail"
    );

    let invalid_orphan_floor = store.run_lifecycle_maintenance(
        &MemoryLifecycleMaintenancePolicy {
            stale_after_unix_ms: 1_000,
            decay_rate: 0.9,
            prune_importance_floor: 0.1,
            orphan_cleanup_enabled: true,
            orphan_importance_floor: 1.1,
            duplicate_cleanup_enabled: true,
            duplicate_similarity_threshold: 0.95,
        },
        10_000,
    );
    assert!(
        invalid_orphan_floor.is_err(),
        "out-of-range orphan_importance_floor must fail"
    );

    let invalid_duplicate_threshold = store.run_lifecycle_maintenance(
        &MemoryLifecycleMaintenancePolicy {
            stale_after_unix_ms: 1_000,
            decay_rate: 0.9,
            prune_importance_floor: 0.1,
            orphan_cleanup_enabled: true,
            orphan_importance_floor: 0.2,
            duplicate_cleanup_enabled: true,
            duplicate_similarity_threshold: 1.1,
        },
        10_000,
    );
    assert!(
        invalid_duplicate_threshold.is_err(),
        "duplicate_similarity_threshold above 1.0 must fail"
    );
}

#[test]
fn spec_2460_c01_lifecycle_maintenance_forgets_noncanonical_near_duplicate_records() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();

    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-canonical", "duplicate lifecycle summary"),
            Some(MemoryType::Fact),
            Some(0.9),
        )
        .expect("write canonical memory");
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-duplicate", "duplicate lifecycle summary"),
            Some(MemoryType::Fact),
            Some(0.2),
        )
        .expect("write duplicate memory");

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: u64::MAX,
                decay_rate: 1.0,
                prune_importance_floor: 0.0,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: true,
                duplicate_similarity_threshold: 0.95,
            },
            20_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(run.duplicate_forgotten_records, 1);

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list latest post dedup");
    assert!(
        listed
            .iter()
            .any(|record| record.entry.memory_id == "memory-canonical"),
        "canonical record should remain active"
    );
    assert!(
        listed
            .iter()
            .all(|record| record.entry.memory_id != "memory-duplicate"),
        "duplicate record should be forgotten"
    );
}

#[test]
fn regression_2460_c02_lifecycle_duplicate_canonical_selection_is_deterministic() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();

    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-alpha", "stable canonical summary"),
            Some(MemoryType::Observation),
            Some(0.4),
        )
        .expect("write memory alpha");
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-beta", "stable canonical summary"),
            Some(MemoryType::Observation),
            Some(0.4),
        )
        .expect("write memory beta");

    let first = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: u64::MAX,
                decay_rate: 1.0,
                prune_importance_floor: 0.0,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: true,
                duplicate_similarity_threshold: 0.95,
            },
            25_000,
        )
        .expect("first lifecycle maintenance run");
    assert_eq!(first.duplicate_forgotten_records, 1);

    let second = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: u64::MAX,
                decay_rate: 1.0,
                prune_importance_floor: 0.0,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: true,
                duplicate_similarity_threshold: 0.95,
            },
            30_000,
        )
        .expect("second lifecycle maintenance run");
    assert_eq!(second.duplicate_forgotten_records, 0);

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list latest post repeated dedup");
    assert_eq!(
        listed.len(),
        1,
        "only canonical active memory should remain"
    );
    assert_eq!(
        listed[0].entry.memory_id, "memory-alpha",
        "canonical selection should remain deterministic across runs"
    );
}

#[test]
fn regression_2460_c03_lifecycle_duplicate_cleanup_skips_identity_records() {
    let temp = tempdir().expect("tempdir");
    let store = FileMemoryStore::new(temp.path());
    let scope = lifecycle_scope();

    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-identity", "stable profile summary"),
            Some(MemoryType::Identity),
            Some(0.9),
        )
        .expect("write identity memory");
    store
        .write_entry_with_metadata(
            &scope,
            lifecycle_entry("memory-fact", "stable profile summary"),
            Some(MemoryType::Fact),
            Some(0.8),
        )
        .expect("write fact memory");

    let run = store
        .run_lifecycle_maintenance(
            &MemoryLifecycleMaintenancePolicy {
                stale_after_unix_ms: u64::MAX,
                decay_rate: 1.0,
                prune_importance_floor: 0.0,
                orphan_cleanup_enabled: false,
                orphan_importance_floor: 0.0,
                duplicate_cleanup_enabled: true,
                duplicate_similarity_threshold: 0.95,
            },
            40_000,
        )
        .expect("run lifecycle maintenance");
    assert_eq!(
        run.duplicate_forgotten_records, 0,
        "identity memories must remain exempt from duplicate cleanup"
    );

    let listed = store
        .list_latest_records(None, usize::MAX)
        .expect("list latest post identity dedup regression");
    assert!(
        listed
            .iter()
            .any(|record| record.entry.memory_id == "memory-identity"),
        "identity memory should remain active"
    );
    assert!(
        listed
            .iter()
            .any(|record| record.entry.memory_id == "memory-fact"),
        "non-identity memory should remain when only duplicate peer is identity"
    );
}

