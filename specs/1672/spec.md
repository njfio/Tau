# Issue 1672 Spec

Status: Implemented

Issue: `#1672`  
Milestone: `#24`  
Parent: `#1661`

## Problem Statement

M24 checkpoint promotion requires a dedicated safety-regression benchmark artifact
that quantifies baseline vs candidate safety deltas and blocks unsafe promotion
with deterministic reason codes.

## Scope

In scope:

- add safety regression benchmark generator script for checkpoint comparisons
- emit report artifact with baseline/trained safety means, delta, threshold, and
  reason codes
- fail closed when safety regression exceeds configured threshold
- integrate benchmark output into live benchmark proof flow

Out of scope:

- external dashboard rendering
- distributed benchmark execution
- algorithm-level reward shaping changes

## Acceptance Criteria

AC-1 (safety benchmark generation):
Given baseline and trained safety sample vectors,
when safety regression benchmark runs,
then report artifact includes baseline/trained means and computed delta.

AC-2 (promotion block on threshold breach):
Given regression above threshold,
when benchmark runs,
then report marks promotion blocked with deterministic reason code and non-zero
exit status.

AC-3 (violation diagnostics):
Given blocked outcome,
when report is inspected,
then threshold and observed violation delta are explicitly present.

AC-4 (integration into eval proof flow):
Given live benchmark proof generation,
when safety benchmark runs,
then proof flow records/uses benchmark output for safety gate decisions.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given valid baseline/trained safety arrays, when benchmark runs, then artifact contains deterministic baseline/trained means and delta fields. |
| C-02 | AC-2 | Regression | Given trained safety regression above threshold, when benchmark runs, then command exits non-zero and reason code `checkpoint_promotion_blocked_safety_regression` is present. |
| C-03 | AC-3 | Functional | Given blocked report, when parsed, then `max_safety_regression` and `safety_regression_delta` are present and positive. |
| C-04 | AC-4 | Integration | Given live benchmark proof run, when executed, then proof artifact references safety benchmark and consumes its promotion decision. |

## Success Metrics

- unsafe checkpoint promotion is blocked by explicit safety benchmark report
- violation deltas are clear and reproducible
- live proof flow consumes safety benchmark output deterministically
