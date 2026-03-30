# Spec: Issue #3642 - Stabilize tau-coding-agent package-scoped tests exposed by fast-validate

Status: Implemented

## Objective
Fix the newly exposed `tau-coding-agent` package-scoped test failures in the
runtime/startup path so `#3631` can progress past validation debt and into the
actual story work.

## Inputs/Outputs
Inputs:
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup/startup_preflight_and_policy.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- targeted failure evidence from the `tau-coding-agent` test suite

Outputs:
- `run_local_runtime` no longer panics by calling a blocking mutex path inside
  async runtime tests.
- tests that temporarily change the process current directory restore it even if
  a runtime assertion fails.
- the currently failing runtime/startup tests pass under package-scoped test
  execution.

## Boundaries / Non-goals
In scope:
- replacing the blocking live-RL snapshot call used during local runtime
  startup
- hardening the relevant test harness around process current-directory changes
- spec artifacts for `#3642`

Out of scope:
- refactoring the full live RL runtime subsystem
- unrelated `tau-coding-agent` test cleanup outside the currently failing path
- broad test-serialization changes across the crate

## Failure Modes
- `run_local_runtime` calls `LiveRlRuntimeBridge::register_if_enabled`, which
  currently returns a snapshot via a blocking mutex path and panics under a
  current-thread Tokio runtime.
- the failing runtime tests change the process current directory and only
  restore it on the success path, so a panic leaves later tests running from a
  deleted temp directory.
- subsequent startup-policy and tool-flow tests then fail with missing cwd,
  file-write, or error-result assertions that cascade from the leaked process
  state.

## Acceptance Criteria
### AC-1 live runtime startup no longer blocks the async runtime thread
Given the current-thread runtime tests for `run_local_runtime`,
when the live RL bridge is registered,
then startup does not call a blocking mutex path from async context.

### AC-2 current-directory test state is restored on failure paths
Given the runtime tests that temporarily switch the process current directory,
when a runtime assertion fails or panics,
then the original directory is still restored before later tests run.

### AC-3 the currently failing tau-coding-agent tests pass
Given the failing package-scoped tests exposed by `fast-validate`,
when the targeted fixes are applied,
then the failing runtime/startup tests pass.

### AC-4 fast-validate advances beyond the tau-coding-agent test blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb` runs,
then it advances beyond the prior `tau-coding-agent` test failure set.

## Conformance Cases
- C-01 / AC-1 / Regression:
  `regression_spec_2542_c03_run_local_runtime_prompt_executes_model_call` and
  `regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent`
  no longer panic with `Cannot block the current thread from within a runtime`.
- C-02 / AC-2 / Regression:
  the startup-policy tests no longer fail with `failed to resolve current
  directory` after the runtime tests run.
- C-03 / AC-3 / Regression:
  the extension/tool-hook runtime tests no longer fail on missing written files
  or unexpected error hook payloads after the shared-state fixes land.
- C-04 / AC-4 / Regression:
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  advances beyond the prior `tau-coding-agent` test failures.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3642/spec.md`
- `specs/3642/plan.md`
- `specs/3642/tasks.md`
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`

## Test Plan
- RED: reproduce the focused failing `tau-coding-agent` tests.
- GREEN: replace the blocking snapshot path and harden current-directory
  restoration in the relevant tests.
- Regression: rerun the focused failing tests and then the exact
  `fast-validate --base <sha>` reproduction.

## Success Metrics / Observable Signals
- the nine currently failing `tau-coding-agent` tests no longer fail in the
  package-scoped run
- any remaining `#3631` failure is beyond the current runtime/startup test set
