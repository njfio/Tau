# Spec: Issue #3665 - Keep TUI client timeout above gateway runtime and provider budgets

Status: Implemented

## Problem Statement
`tau-unified.sh tui` currently forwards the same `--request-timeout-ms` value to
both the gateway/runtime bootstrap path and the interactive `tau-tui` HTTP
client. That is not enough for the Ralph-loop gateway path because one user turn
can consume the first full provider/runtime budget plus bounded verifier-driven
retry attempts. The result is a client-side transport error before the gateway
can surface a structured failure or success.

## Scope
In scope:
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `specs/3665/spec.md`
- `specs/3665/plan.md`
- `specs/3665/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing Rust-side `tau-tui` timeout defaults
- changing gateway retry policy or verifier semantics
- redesigning direct `tau-tui interactive` invocation outside the unified launcher

## Acceptance Criteria
### AC-1 Interactive TUI client timeout exceeds gateway/runtime timeout budget
Given `tau-unified.sh tui` is invoked for interactive gateway mode,
when it launches `tau-tui`,
then it passes a client timeout strictly greater than the runtime/provider
request timeout budget so the client does not disconnect first during bounded
Ralph-loop retries.

### AC-2 Runtime bootstrap timeout stays unchanged
Given `tau-unified.sh tui --bootstrap-runtime` is invoked,
when it starts `tau-coding-agent`,
then the runtime/provider timeout budget remains the operator-specified
`--request-timeout-ms` value and is not inflated to the client-side timeout.

### AC-3 Launcher tests cover default and override separation
Given the launcher shell tests run,
when they inspect the recorded `tui` and `up` commands,
then they prove the interactive client timeout is derived above the runtime
budget for both defaults and explicit overrides.

## Conformance Cases
- C-01 / AC-1 / Functional:
  default interactive `tui` launch records a client timeout greater than the
  default runtime/provider timeout.
- C-02 / AC-2 / Functional:
  bootstrap `up` still records the operator-specified runtime/provider timeout.
- C-03 / AC-3 / Regression:
  explicit `--request-timeout-ms` override shows a larger interactive client
  timeout while runtime/provider flags stay on the base override.
- C-04 / AC-3 / Regression:
  existing launcher tests remain green after the timeout split.

## Success Metrics / Observable Signals
- Interactive TUI turns prefer structured gateway failures or completions over
  `error sending request for url` transport aborts caused by the client timing
  out first.
- Launcher logs and runner captures show distinct interactive client and
  runtime/provider timeout budgets.

## Files To Touch
- `specs/3665/spec.md`
- `specs/3665/plan.md`
- `specs/3665/tasks.md`
- `specs/milestones/m334/index.md`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
