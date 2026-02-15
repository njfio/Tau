# Issue 1654 Spec

Status: Implemented

Issue: `#1654`  
Milestone: `#23`  
Parent: `#1624`

## Problem Statement

M23 remains below the required `>=3,000` rustdoc marker threshold. Wave 2 crates
(`tau-multi-channel`, `tau-gateway`, `tau-provider`, `tau-ops`) still contain a
large set of undocumented public APIs, creating maintainability risk and
blocking milestone exit.

## Scope

In scope:

- capture baseline undocumented-public API hotspots for the 4 wave-2 crates
- add rustdoc `///` comments for undocumented public items in scope crates
- regenerate marker-count and threshold artifacts to show delta and milestone
  gate status
- run crate-level tests for touched crates to ensure no behavior regressions

Out of scope:

- semantic behavior changes to runtime/provider logic
- changing marker counting scripts or threshold policy semantics

## Acceptance Criteria

AC-1 (baseline evidence):
Given wave-2 crates,
when hotspot scan runs,
then baseline JSON/Markdown artifacts record undocumented public item counts.

AC-2 (documentation uplift):
Given undocumented public items in wave-2 crates,
when implementation completes,
then those items have rustdoc `///` comments documenting API intent/contract.

AC-3 (threshold achievement):
Given M23 threshold target `3000`,
when marker verification runs after uplift,
then artifacts report `meets=true` and `remaining=0`.

AC-4 (regression safety):
Given touched crates,
when crate test suites run,
then tests pass for `tau-multi-channel`, `tau-gateway`, `tau-provider`,
and `tau-ops`.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | Given baseline scan command, when executed, then `m23-wave2-undocumented-hotspots-baseline.{json,md}` are generated. |
| C-02 | AC-2 | Functional | Given wave-2 crates, when scan runs after edits, then undocumented public count is reduced to zero in scope crates. |
| C-03 | AC-3 | Conformance | Given threshold verification command, when run after uplift, then `meets=true` and `remaining=0` are emitted. |
| C-04 | AC-4 | Regression | Given crate test commands, when run for touched crates, then all pass. |

## Success Metrics

- M23 marker total reaches or exceeds `3000`
- all wave-2 scope crates show complete documented-public coverage
- reproducible artifacts capture baseline and after state
