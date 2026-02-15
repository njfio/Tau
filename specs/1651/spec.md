# Issue 1651 Spec

Status: Implemented

Issue: `#1651`  
Milestone: `#23`  
Parent: `#1622`

## Problem Statement

M23 gate work requires a deterministic command that counts raw rustdoc markers
(`///` and `//!`) across workspace crates, with per-crate breakdown and
machine-readable output. Current doc-density tooling measures documented public
API coverage percent, not total marker counts needed for `>= 3,000` tracking.

## Scope

In scope:

- add checked-in command for raw rustdoc marker counting
- emit stable per-crate and total counts in JSON output
- provide human-readable report output for operators
- add script tests for functional and regression behavior
- document command in docs runbook

Out of scope:

- increasing marker count to `>= 3,000`
- CI workflow rewiring
- comment quality scoring heuristics

## Acceptance Criteria

AC-1 (count command):
Given repository source,
when marker-count command runs,
then it reports total marker count and per-crate counts.

AC-2 (machine-readable output):
Given marker-count command run with JSON output path,
when execution completes,
then JSON artifact includes schema version, generated timestamp, repo root,
total markers, and sorted crate breakdown.

AC-3 (deterministic behavior and docs wiring):
Given repeated runs on unchanged input,
when script is executed,
then counts remain stable and docs reference command usage.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given fixture crates with known doc-marker lines, when script runs, then total/per-crate counts match expected values. |
| C-02 | AC-2 | Conformance | Given JSON output path, when script runs, then artifact contains required fields with stable schema. |
| C-03 | AC-3 | Regression | Given missing crates dir or unknown flag, when script runs, then command fails non-zero with deterministic error message. |
| C-04 | AC-3 | Integration | Given docs scorecard, when reviewed, then marker-count command usage is documented. |

## Success Metrics

- one-command reproducible marker-count evidence for M23 tracking
- script contract tests pass for fixture and error paths
- docs scorecard includes command and artifact usage guidance
