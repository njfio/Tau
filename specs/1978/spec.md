# Issue 1978 Spec

Status: Accepted

Issue: `#1978`  
Milestone: `#24`  
Parent: `#1662`

## Problem Statement

Benchmark gate reports can be built in-memory, but there is no deterministic
export and replay-validation helper pair for persisting/reporting quality gate
results in automation workflows.

## Scope

In scope:

- add deterministic gate report export helper
- add replay validator helper for exported gate report JSON
- reuse machine-readable report structure with required `manifest` + `quality`
  sections

Out of scope:

- CI workflow wiring
- remote storage/object-store transport
- dashboard rendering

## Acceptance Criteria

AC-1 (deterministic export):
Given a valid gate report and output directory,
when export runs,
then deterministic file path and bytes-written summary are returned.

AC-2 (validator pass path):
Given exported gate report JSON,
when validator runs,
then payload is accepted and required top-level sections are present.

AC-3 (validator fail closed):
Given malformed or non-object gate report JSON,
when validator runs,
then deterministic validation errors are returned.

AC-4 (invalid destination fail closed):
Given export destination that is a file path,
when export runs,
then deterministic directory error is returned.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given valid gate report and empty directory, when export runs, then deterministic filename is written and bytes_written > 0. |
| C-02 | AC-2 | Conformance | Given exported gate report, when validator runs, then payload contains `manifest` and `quality` objects. |
| C-03 | AC-3 | Unit | Given malformed/non-object JSON, when validator runs, then deterministic parse/object validation errors are returned. |
| C-04 | AC-4 | Regression | Given export destination path that is a file, when export runs, then deterministic directory error is returned. |

## Success Metrics

- gate report can be persisted and replay-validated with deterministic helpers
- exported payload remains machine-readable for automation consumers
- fail-closed behavior prevents silent acceptance of malformed evidence
