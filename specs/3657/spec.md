# Spec: Issue #3657 - Add structured verifier bundles and first back-pressure adapters to gateway missions

Status: Reviewed

## Problem Statement
`#3655` introduced a mission supervisor record and `#3656` wired mission runs
into Tau's learning surfaces, but the gateway outer loop still makes its
continue/stop decision from a single tool-evidence verifier. That is not enough
for a Ralph-style loop. Workspace-changing requests can stop after any tool
execution, even if no mutating work happened, and prompts that explicitly ask
for validation can stop without any successful validation signal. The gateway
needs a structured verifier bundle with first-class back-pressure adapters so
the outer loop keeps iterating until stronger completion evidence is present or
the retry budget is exhausted.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/verifier_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3657/spec.md`
- `specs/3657/plan.md`
- `specs/3657/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- operator mission-control UI changes
- adding a new public HTTP request/response schema
- changing the inner `tau-agent-core` tool loop contract
- full task-plan/checkpoint orchestration across `tau-orchestrator`

## Acceptance Criteria
### AC-1 Gateway mission iterations persist structured verifier bundles
Given a gateway mission iteration,
when verifier evaluation runs,
then the iteration persists a structured verifier bundle with an overall verdict
plus the individual verifier records that produced the decision.

### AC-2 Workspace-changing prompts continue until mutating work is observed
Given an action-oriented gateway prompt that requests creating, editing,
fixing, or otherwise mutating workspace state,
when an iteration performs tool work without any successful mutating evidence,
then the outer loop continues with corrective verifier feedback instead of
stopping as complete.

### AC-3 Validation-requesting prompts continue until successful validation evidence is observed
Given a gateway prompt that explicitly asks for testing, validation,
verification, or playability evidence,
when an iteration lacks a successful validation signal,
then the outer loop continues until validation evidence is observed or the
retry budget is exhausted and the mission blocks.

## Conformance Cases
- C-01 / AC-1 / Unit:
  build a verifier bundle from mixed tool traces and verify it records the
  individual tool-evidence, mutation-evidence, and validation-evidence
  verifiers with the correct overall status.
- C-02 / AC-2 / Regression:
  run a workspace-changing prompt where the first attempt only uses non-mutating
  tool activity, then verify the gateway retries and only completes after a
  later attempt produces successful mutating evidence.
- C-03 / AC-3 / Regression:
  run a prompt that explicitly asks for validation, let an early attempt mutate
  files without validation, then verify the gateway retries until a successful
  validation signal is observed and records the validation verifier state.

## Success Metrics / Observable Signals
- Mission iteration state exposes more than a single tool-evidence reason code
- Workspace-changing requests no longer stop after irrelevant tool activity
- Validation-requesting prompts surface continuous verifier back-pressure before
  mission completion

## Files To Touch
- `specs/3657/spec.md`
- `specs/3657/plan.md`
- `specs/3657/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/verifier_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
