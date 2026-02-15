# Issue 1652 Spec

Status: Implemented

Issue: `#1652`  
Milestone: `#23`  
Parent: `#1622`

## Problem Statement

M23 requires a quantitative allocation plan that maps baseline documentation
markers (`1,438`) to a reproducible path toward `>=3,000`. Existing threshold
targets do not define crate-level marker quotas, owners, or checkpoint totals.

## Scope

In scope:

- define crate-level marker quota table for high-impact crates
- publish checkpoint totals and dates toward `>=3,000`
- commit machine-readable allocation artifact and human-readable summary
- validate allocation math and contract fields with tests

Out of scope:

- performing all documentation uplift work to reach `>=3,000`
- modifying CI workflow behavior

## Acceptance Criteria

AC-1 (allocation table):
Given current marker report,
when allocation plan is authored,
then per-crate `current/target/delta` quotas are explicit.

AC-2 (target math):
Given allocation plan,
when summing deltas,
then target total markers is `>=3,000` and math is internally consistent.

AC-3 (checkpoint publication):
Given M23 plan,
when operators review roadmap docs,
then checkpoint dates and minimum totals are documented.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given `m23-doc-allocation-plan.json`, when read, then quota rows include `crate/current/target/delta`. |
| C-02 | AC-2 | Conformance | Given plan JSON, when validating totals, then `current + sum(delta) = target` and `target >= 3000`. |
| C-03 | AC-3 | Integration | Given docs guides, when reviewed, then checkpoint schedule and plan references are discoverable. |
| C-04 | AC-2, AC-3 | Regression | Given contract tests, when run, then allocation schema and docs links remain valid. |

## Success Metrics

- allocation plan artifact committed and reproducible
- target path reaches at least `3,000` markers on paper
- checkpoint schedule and ownership metadata are discoverable in docs
