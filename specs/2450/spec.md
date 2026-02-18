# Spec #2450 - add lifecycle metadata fields and forgotten filtering in memory runtime

Status: Implemented
Milestone: specs/milestones/m76/index.md
Issue: https://github.com/njfio/Tau/issues/2450

## Problem Statement

Tau memory runtime currently cannot track access lifecycle information and does
not support retaining-but-hiding soft-deleted memories.

## Scope

In scope:

- Extend runtime memory records with:
  - `last_accessed_at_unix_ms`
  - `access_count`
  - `forgotten`
- Ensure legacy records deserialize safely with defaults.
- Add runtime soft-delete API and default forgotten filtering in
  read/list/search/tree pathways.
- Add memory tool `memory_delete` to expose soft-delete operation.

Out of scope:

- Scheduler-driven decay/pruning jobs.
- Near-duplicate merge and orphan cleanup logic.

## Acceptance Criteria

- AC-1: Given legacy records without lifecycle fields, when deserialized, then
  defaults are stable (`last_accessed_at_unix_ms=0`, `access_count=0`,
  `forgotten=false`).
- AC-2: Given active records are returned by read/search, when operation
  succeeds, then lifecycle access metadata is updated.
- AC-3: Given a soft-delete request, when `memory_delete` succeeds, then record
  is marked forgotten and excluded from default read/search/list/tree output.
- AC-4: Given unknown memory id delete requests, when `memory_delete` executes,
  then deterministic not-found payload is returned.

## Conformance Cases

- C-01 (AC-1, unit): runtime record serde defaults for lifecycle fields.
- C-02 (AC-2, integration): read/search updates `last_accessed_at_unix_ms` and
  increments `access_count`.
- C-03 (AC-3, functional): `memory_delete` marks forgotten and subsequent
  read/search/list/tree exclude record.
- C-04 (AC-4, functional): `memory_delete` not-found returns reason code
  `memory_not_found`.

## Success Metrics / Observable Signals

- C-01..C-04 pass.
- `cargo fmt --check` and scoped `clippy` pass.
- `tau-memory` and lifecycle-focused `tau-tools` tests pass.
