# Spec #2045

Status: Implemented
Milestone: specs/milestones/m25/index.md
Issue: https://github.com/njfio/Tau/issues/2045

## Problem Statement

M25.4 requires a task-level build/test latency baseline artifact with ranked
hotspots to guide fast-lane and budget-enforcement follow-up work. Without this
roll-up baseline, subsequent tuning tasks (`#2046`, `#2047`, `#2048`) cannot
measure improvement or regression against a shared source of truth.

## Acceptance Criteria

- AC-1: Baseline artifacts are checked in as JSON + Markdown under
  `tasks/reports/` with command-level timing stats.
- AC-2: Hotspots are explicitly ranked by average duration and include
  reproducible command invocations.
- AC-3: Baseline generation and contract validation tests pass with both
  deterministic fixture mode and real live command execution evidence.

## Scope

In:

- Consume and finalize merged `#2068` baseline pipeline deliverables.
- Record a live baseline artifact capture for M25.4.1.
- Verify shell + Python conformance tests and link evidence.

Out:

- Fast-lane wrapper implementation (`#2046`/`#2069`).
- CI cache/parallel optimization (`#2047`/`#2070`).
- Regression budget policy enforcement (`#2048`/`#2071`).

## Conformance Cases

- C-01 (AC-1, functional): `tasks/reports/m25-build-test-latency-baseline.json`
  and `.md` exist with non-empty command rows.
- C-02 (AC-2, integration): hotspot ordering in JSON is descending by `avg_ms`
  and Markdown includes command invocation rows.
- C-03 (AC-3, regression): `scripts/dev/test-build-test-latency-baseline.sh`
  passes, including fail-closed invalid-fixture behavior.
- C-04 (AC-3, contract): Python contract test
  `.github/scripts/test_build_test_latency_baseline_contract.py` passes.

## Success Metrics

- Live baseline artifact generated and committed with environment metadata +
  ranked command timings.
- Conformance suite passes for all C-01..C-04 cases.
- `#2045` is closed with links to PR evidence and parent-story handoff.
