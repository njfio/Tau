# Spec #2070

Status: Implemented
Milestone: specs/milestones/m25/index.md
Issue: https://github.com/njfio/Tau/issues/2070

## Problem Statement

M25.4.3 requires CI runtime reductions without reducing required validation
coverage. The current workflow uses generic rust-cache configuration and serial
execution for helper Python suites. We need explicit cache-key tuning and
parallel helper scheduling with measurable loop-time improvement evidence.

## Acceptance Criteria

- AC-1: CI workflow rust-cache steps use explicit, lane-scoped shared keys for
  Linux quality, WASM smoke, cross-platform smoke, and coverage lanes.
- AC-2: Helper Python suite scheduling is parallelized while preserving required
  test coverage (same discovery scope/pattern).
- AC-3: A timing comparison artifact documents median serial vs parallel helper
  loop durations, with observed improvement status and diagnostics.

## Scope

In:

- Update `.github/workflows/ci.yml` cache-key and helper test scheduling.
- Add deterministic timing comparison script/report for helper suite.
- Add shell + Python contract tests validating workflow snippet requirements and
  report shape.
- Add operator guide for tuning and measurement workflow.

Out:

- Task-level roll-up closure for `#2047`.
- Broad CI architecture redesign unrelated to cache/scheduling tuning.

## Conformance Cases

- C-01 (AC-1, integration): CI workflow contains explicit rust-cache shared-key
  configuration for all four rust-cache lanes.
- C-02 (AC-2, functional): helper test step runs the parallel helper runner with
  unchanged discovery scope/pattern (`.github/scripts`, `test_*.py`).
- C-03 (AC-3, functional): comparison JSON + Markdown reports include serial and
  parallel medians plus improvement status.
- C-04 (AC-2/AC-3, regression): malformed timing fixture fails closed with
  non-zero exit and actionable error.

## Success Metrics

- Workflow contract tests remain green with no loss of required checks.
- Timing report artifact shows measurable helper-loop median improvement.
- `#2070` closes with validated cache/scheduling tuning evidence.
