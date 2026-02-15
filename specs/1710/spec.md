# Issue 1710 Spec

Status: Implemented

Issue: `#1710`  
Milestone: `#24`  
Parent: `#1663`

## Problem Statement

M24 needs a reproducible, operator-facing proof that RL operational controls and
safety promotion constraints are functioning together. Today these checks exist
in separate places, but there is no single proof artifact flow that runs them
as one safety gate.

## Scope

In scope:

- add a deterministic proof script that runs operational control and safety gate
  checks
- emit machine-readable and markdown artifacts for operator audit
- validate operator runbook evidence path exists in proof output
- add script-level tests for pass/fail execution and artifact schema

Out of scope:

- new RL optimization algorithm behavior
- remote CI orchestration changes
- dashboard UI changes

## Acceptance Criteria

AC-1 (operational control proof):
Given a proof run,
when the script executes control-plane checks,
then pause/resume/rollback control coverage command is executed and recorded.

AC-2 (safety constraints proof):
Given a proof run,
when the script executes safety checks,
then checkpoint-promotion safety gate coverage command is executed and recorded.

AC-3 (operator evidence artifact):
Given a proof run,
when execution completes,
then JSON/Markdown artifacts contain per-check status, command, duration, and
runbook evidence path state.

AC-4 (fail-closed proof gate):
Given any failing proof check,
when the script completes,
then overall status is `fail` and process exits non-zero.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given a passing runner hook, when proof script executes, then control check id appears with pass status in JSON artifact. |
| C-02 | AC-2 | Functional | Given a passing runner hook, when proof script executes, then safety-gate check id appears with pass status in JSON artifact. |
| C-03 | AC-3 | Integration | Given proof execution, when JSON/MD artifacts are parsed, then schema fields, per-check timings, and runbook evidence fields exist. |
| C-04 | AC-4 | Regression | Given a failing runner hook for one check, when proof script executes, then exit code is non-zero and overall status is `fail` with failing check recorded. |

## Success Metrics

- one command produces consistent operational-safety proof artifacts
- artifact schema is test-enforced
- failure paths fail closed with deterministic evidence
