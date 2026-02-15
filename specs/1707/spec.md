# Issue 1707 Spec

Status: Implemented

Issue: `#1707`  
Milestone: `#23`  
Parent: `#1701`

## Problem Statement

M23 requires a reproducible gate check for raw rustdoc markers with crate-level
breakdown and explicit threshold status (`>=3,000`). Existing marker counting
artifacts do not include baseline-vs-current deltas in one gate report.

## Scope

In scope:

- produce updated marker-count artifacts from `scripts/dev/rustdoc-marker-count.sh`
- add threshold verification artifact with baseline/current totals and per-crate deltas
- make threshold pass/fail status explicit for milestone gate reviews

Out of scope:

- forcing marker total above threshold in this issue
- changing marker counting semantics

## Acceptance Criteria

AC-1 (reproducible current snapshot):
Given repo state,
when marker count command runs,
then JSON/Markdown artifacts capture current total and per-crate counts.

AC-2 (baseline vs current deltas):
Given baseline and current artifacts,
when threshold verification runs,
then output includes baseline/current totals and per-crate deltas.

AC-3 (explicit gate status):
Given threshold target 3000,
when verification runs,
then artifact records pass/fail and remaining gap (`max(0, threshold-current)`).

AC-4 (regression safety):
Given script tests,
when validation checks run,
then threshold verification contracts pass.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given marker-count command, when run, then current total/per-crate artifacts are produced. |
| C-02 | AC-2 | Conformance | Given baseline+current artifacts, when verification command runs, then per-crate delta rows are emitted. |
| C-03 | AC-3 | Conformance | Given threshold=3000, when current total is below threshold, then artifact marks `meets_threshold=false` and reports remaining gap. |
| C-04 | AC-4 | Regression | Given verification script tests, when executed, then all pass. |

## Success Metrics

- current marker snapshot regenerated with reproducible command
- threshold verification artifact published with baseline/current delta matrix
