# Spec: Issue #3429 - Integrated cross-stream reliability and recovery proof

Status: Implemented

## Problem Statement
Current verification is split across stream-specific checks, but integrated reliability evidence for restart/reconnect/degraded-dependency behavior is not consolidated into a deterministic suite with machine-readable pass/fail reporting. This makes cross-stream promotion riskier and slower to audit.

## Scope
In scope:
- Add a dedicated integrated reliability verification suite for M296 that runs cross-stream checks for restart recovery, reconnect behavior, degraded dependency handling, auth fail-closed behavior, and RL conformance.
- Add deterministic test coverage for the verification suite itself (pass/fail/report shape) using mocked command execution.
- Produce machine-readable report artifacts for CI/operator consumption.

Out of scope:
- New gateway API/wire contract fields.
- New RL algorithms or trainer internals.
- External infrastructure/chaos tooling integration.

## Acceptance Criteria
### AC-1 Integrated reliability suite executes required cross-stream checks
Given M296 reliability verification is invoked,
when the suite runs,
then it executes restart, reconnect, degraded dependency, auth fail-closed, and RL checks in one run with per-step status.

### AC-2 Reliability suite report is deterministic and machine-readable
Given the integrated suite finishes,
when report output is written,
then JSON report includes schema/version, suite id, overall status, and ordered step results with log paths.

### AC-3 Suite is fail-closed and preserves failure evidence
Given one or more reliability steps fail,
when the suite exits,
then overall result is `fail`, failing step(s) remain in report, and process exits non-zero.

### AC-4 Spec/process artifacts and verification evidence are complete
Given AGENTS contract requirements,
when artifacts are reviewed,
then `spec.md`, `plan.md`, and `tasks.md` exist with AC/conformance/test mappings and RED/GREEN/REGRESSION evidence.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | integrated suite invocation | run suite | executes required cross-stream step ids |
| C-02 | AC-2 | Functional/Conformance | successful mocked command execution | run suite | report JSON contains deterministic schema/suite/steps payload |
| C-03 | AC-3 | Regression/Conformance | one mocked step fails | run suite | overall `fail`, failing step recorded, non-zero exit |
| C-04 | AC-4 | Conformance | issue artifact set | verify paths/sections | spec/plan/tasks exist with evidence mapping |

## Success Metrics / Observable Signals
- Single command emits integrated reliability result across restart/reconnect/degraded/auth/RL checks.
- Report format remains stable and CI-friendly.
- Self-test harness catches pass/fail/report regressions deterministically.
