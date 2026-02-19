# Spec #2584 - Task: execute G5-G7 conformance audit and update roadmap checklist items

Status: Reviewed
Priority: P1
Milestone: M100
Parent: #2583

## Problem Statement
`tasks/spacebot-comparison.md` still marks G5/G6/G7 memory-enhancement pathways as incomplete, while `tau-memory` and `tau-tools` already appear to implement typed memories, relation graph features, and lifecycle maintenance. We need explicit conformance validation and checklist reconciliation.

## Scope
- Map G5/G6/G7 checklist bullets to concrete code/tests.
- Run targeted conformance/regression tests across `tau-memory` and `tau-tools`.
- Update roadmap checklist statuses where criteria are verified.
- Capture process evidence for closure.

## Out of Scope
- Net-new memory model architecture changes.
- Large behavior rewrites unrelated to checklist parity.

## Acceptance Criteria
- AC-1: G5 criteria (typed memories + importance scoring + search boost + tool args) are validated with passing evidence.
- AC-2: G6 criteria (relations schema + `relates_to` ingestion + graph-aware search scoring) are validated with passing evidence.
- AC-3: G7 criteria (access metadata + decay/prune/orphan/forgotten lifecycle behavior) are validated with passing evidence.
- AC-4: `tasks/spacebot-comparison.md` G5/G6/G7 checkboxes are updated to match validated implementation status.

## Conformance Cases
- C-01 (AC-1, conformance): `cargo test -p tau-memory unit_memory_type_default_importance_profile_and_record_defaults -- --test-threads=1`
- C-02 (AC-1, conformance): `cargo test -p tau-tools "spec_c0[1-4]_memory_" -- --test-threads=1`
- C-03 (AC-2, conformance): `cargo test -p tau-tools spec_2444_ -- --test-threads=1`
- C-04 (AC-2, functional): `cargo test -p tau-memory integration_search_score_uses_vector_importance_and_graph_signal_additively -- --test-threads=1`
- C-05 (AC-3, regression): `cargo test -p tau-memory "spec_2455_|spec_2450_c02" -- --test-threads=1`
- C-06 (AC-4, process): roadmap checklist updated for G5/G6/G7 with validated status

## Success Signals
- G5/G6/G7 status reflects actual implementation, not stale checklist state.
- Evidence is reproducible from commands listed in this spec.
