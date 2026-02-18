# Spec #2448 - G7 memory lifecycle phase-1 orchestration

Status: Accepted
Milestone: specs/milestones/m76/index.md
Issue: https://github.com/njfio/Tau/issues/2448

## Problem Statement

Tau needs memory lifecycle controls from `G7` so memory growth and relevance do
not degrade over time. This epic orchestrates the first lifecycle slice.

## Scope

In scope:

- Lifecycle metadata tracking (`last_accessed_at_unix_ms`, `access_count`).
- Soft-delete semantics (`forgotten`) and default search/read/list exclusion.
- Tooling path for memory soft-delete.

Out of scope:

- Decay scheduler/heartbeat pruning.
- Near-duplicate merge and orphan cleanup jobs.

## Acceptance Criteria

- AC-1: Story/task/subtask hierarchy lands with spec-driven artifacts.
- AC-2: Task #2450 implementation merges with conformance and regression
  coverage.
- AC-3: Epic closure includes AC-to-test traceability and quality gate evidence.

## Conformance Cases

- C-01 (AC-1): specs for #2448/#2449/#2450/#2451 exist and link milestone.
- C-02 (AC-2): #2450 conformance tests pass for lifecycle metadata and forgotten
  filtering.
- C-03 (AC-3): closure notes include verify commands and test-tier outcomes.
