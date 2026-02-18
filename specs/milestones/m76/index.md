# M76 - Spacebot G7 Memory Lifecycle (Phase 1)

Milestone: GitHub milestone `M76 - Spacebot G7 Memory Lifecycle (Phase 1)`

## Objective

Deliver the first production slice of `tasks/spacebot-comparison.md` gap `G7`
by adding lifecycle metadata tracking and soft-delete behavior to Tau memory.

## Scope

- Add lifecycle metadata fields on runtime memory records:
  - `last_accessed_at_unix_ms`
  - `access_count`
- Update read/search pathways to advance lifecycle metadata for returned records.
- Add soft-delete flag (`forgotten`) and exclude forgotten records from default
  search/list/read responses.
- Add memory tool support for lifecycle deletion flow (`memory_delete`) with
  deterministic reason codes.
- Add conformance + regression tests for lifecycle metadata and soft-delete
  behavior.

## Out of Scope

- Time-decay scheduler/heartbeat pruning.
- Near-duplicate merge pipeline.
- Orphan graph cleanup jobs.
- UI lifecycle dashboards.

## Linked Hierarchy

- Epic: #2448
- Story: #2449
- Task: #2450
- Subtask: #2451
