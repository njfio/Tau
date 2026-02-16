# Issue 1627 Spec

Status: Implemented

Issue: `#1627`  
Milestone: `#21`  
Parent: `#1610`

## Problem Statement

Scaffold merge/remove work is blocked by ambiguous decision criteria and missing
candidate-by-candidate scoring artifacts. A deterministic rubric and scoring
sheet are required so each surface has an explicit keep/merge/remove action,
owner, and rationale.

## Scope

In scope:

- publish merge/remove decision rubric with objective scoring criteria
- generate machine-readable and markdown scoring sheet artifacts
- score every scaffold candidate surface in the lane with explicit action
- fail closed when candidate metadata is invalid or unresolved decisions remain

Out of scope:

- executing merge/remove code changes for selected candidates
- CI workflow wiring changes
- dependency additions

## Acceptance Criteria

AC-1 (rubric policy):
Given the lane’s scaffold decision criteria,
when the rubric artifact is published,
then operator value, runtime usage, maintenance cost, and test posture are
defined with scoring rules and action thresholds.

AC-2 (full scoring sheet):
Given the candidate surface list,
when the scoring sheet is generated,
then each candidate has explicit owner, keep/merge/remove action, and rationale
with no unresolved decisions.

AC-3 (deterministic outputs):
Given fixed generated timestamp and candidate input,
when the generator runs repeatedly,
then JSON/Markdown outputs are machine-readable and deterministic.

AC-4 (fail-closed regression):
Given invalid candidate metadata or unresolved decision state,
when the generator runs,
then deterministic validation errors are returned.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given rubric policy JSON, when loaded, then required criteria and thresholds exist and are typed. |
| C-02 | AC-2 | Functional | Given default candidate set, when generator runs, then each candidate has owner/action/rationale and unresolved count is zero. |
| C-03 | AC-3 | Conformance | Given fixed timestamp, when generator runs twice, then JSON summary and markdown sections are deterministic. |
| C-04 | AC-4 | Regression | Given invalid fixture data (missing owner or unresolved action), when generator runs, then deterministic fail-closed errors are emitted. |

## Success Metrics

- all candidate surfaces in this lane have explicit keep/merge/remove actions
- scoring artifacts are reproducible and machine-readable
- invalid or ambiguous decision inputs are blocked deterministically
