# Issue 1698 Spec

Status: Implemented

Issue: `#1698`  
Milestone: `#24`  
Parent: `#1662`

## Problem Statement

M24 requires reproducible live-run RL benchmark proof, but there is no standard
protocol or artifact contract for maintainers to produce comparable baseline vs
trained evidence across runs.

## Scope

In scope:

- publish a documented live-run benchmark protocol in `docs/guides/training-ops.md`
- define required benchmark proof artifacts and pass/fail significance criteria
- provide a reusable artifact template under `scripts/demo/`
- add validator + regression test for artifact template contract

Out of scope:

- implementing the benchmark fixture generator itself (`#1697`)
- implementing significance computation engine (`#1674`)

## Acceptance Criteria

AC-1 (protocol published):
Given `training-ops.md`,
when maintainers follow the guide,
then baseline/trained live-run benchmark workflow is reproducible.

AC-2 (artifact template contract):
Given `scripts/demo` benchmark template,
when validated,
then required fields and pass/fail criteria structure are enforced.

AC-3 (pass/fail criteria defined):
Given benchmark proof docs/template,
when reviewed,
then explicit significance and regression thresholds are present.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given the guide, when run steps are executed, then baseline/trained artifact paths and commands are explicit. |
| C-02 | AC-2 | Regression | Given template JSON, when validator runs, then required keys and threshold fields pass/fail deterministically. |
| C-03 | AC-3 | Functional | Given protocol docs, when reviewed, then significance confidence/p-value and regression gate thresholds are documented. |

## Success Metrics

- maintainers can produce comparable M24 proof artifacts with one documented protocol
