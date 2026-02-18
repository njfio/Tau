# Spec #2460 - add G7 phase-3 dedup + heartbeat lifecycle orchestration

Status: Implemented
Milestone: specs/milestones/m78/index.md
Issue: https://github.com/njfio/Tau/issues/2460

## Problem Statement

The lifecycle maintenance API handles decay/prune/orphan cleanup, but still
lacks near-duplicate suppression and is not executed automatically by runtime
heartbeat.

## Scope

In scope:

- Extend lifecycle policy with duplicate-detection controls.
- Soft-forget duplicate non-canonical records.
- Wire heartbeat cycle to invoke lifecycle maintenance.
- Capture lifecycle diagnostics/reason codes in heartbeat outputs.

Out of scope:

- New external model dependencies.

## Acceptance Criteria

- AC-1: Given similar active memories above threshold, maintenance run marks
  duplicate non-canonical records forgotten.
- AC-2: Canonical record selection is deterministic and stable.
- AC-3: Heartbeat cycle executes lifecycle maintenance when configured and
  reports lifecycle counters.
- AC-4: Maintenance failures are surfaced as diagnostics/reason codes and
  heartbeat cycle completes.

## Conformance Cases

- C-01 (AC-1, integration): duplicate similarity threshold path forgets one
  duplicate record.
- C-02 (AC-2, regression): deterministic canonical winner remains active across
  repeated maintenance runs.
- C-03 (AC-3, integration): heartbeat cycle includes lifecycle diagnostics after
  successful maintenance run.
- C-04 (AC-4, regression): heartbeat cycle records lifecycle failure reason code
  when maintenance setup is invalid.

## Success Metrics / Observable Signals

- C-01..C-04 pass.
- Existing lifecycle phase-1/phase-2 tests remain green.
- Heartbeat scheduler tests remain green.
