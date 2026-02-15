# Issue 1740 Spec

Status: Accepted

Issue: `#1740`  
Milestone: `#24`  
Parent: `#1663`

## Problem Statement

M24 requires proof that RL/optimization job operations can recover after a
process crash and that operators have a reproducible playbook artifact that
captures commands, logs, and resume outcomes.

## Scope

In scope:

- add runtime integration coverage for restart recovery of in-flight jobs
- define a validated operator playbook artifact contract for resume-after-crash
- document the drill steps and required evidence in training operations guide

Out of scope:

- introducing new training algorithms
- adding new runtime dependencies
- changing existing RPC wire formats

## Acceptance Criteria

AC-1 (controlled crash + resume):
Given a persisted running job manifest,
when runtime restarts,
then runtime recovers the job into queue and executes it to terminal success.

AC-2 (operator playbook artifact):
Given a resume-after-crash drill,
when operators capture evidence,
then a playbook artifact contains required commands, state paths, and log paths.

AC-3 (compliance validation):
Given playbook artifacts,
when validation runs,
then malformed artifacts fail with deterministic error output.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Integration | Given a running manifest at restart, when runtime initializes, then it emits recovery evidence and the recovered job reaches terminal success. |
| C-02 | AC-2 | Functional | Given a playbook artifact template, when populated with valid fields, then validator accepts it. |
| C-03 | AC-3 | Regression | Given malformed/missing crash or resume evidence fields, when validator runs, then it fails closed. |

## Success Metrics

- restart recovery behavior has deterministic integration proof
- operators can produce and validate a standard M24 resume-after-crash artifact
