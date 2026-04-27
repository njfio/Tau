# Spec: Issue #3650 - Fail just tui closed when runtime bootstrap is unavailable or unready

Status: Reviewed

## Problem Statement
`just tui` delegates to `scripts/run/tau-unified.sh tui`, which can bootstrap the unified runtime and then launch the interactive TUI even when the runtime never becomes reachable or dashboard readiness artifacts never appear. If the target bind is already occupied by an unmanaged process, or if the runtime exits during bootstrap, the operator can land in a shell connected to a dead or wrong gateway and only see opaque connection errors later.

## Scope
In scope:
- `scripts/run/tau-unified.sh` runtime bootstrap behavior for `up` and `tui`.
- `scripts/run/test-tau-unified.sh` deterministic shell regressions for bind-conflict and readiness failure paths.
- `just tui` behavior through the unified launcher path.
- Preserving runner-mode coverage so launcher tests remain fast and deterministic.

Out of scope:
- Redesigning the TUI UI.
- Changing gateway request timeout or provider retry policy.
- Starting a second launcher or bypassing `scripts/run/tau-unified.sh`.
- Masking bootstrap failures inside tau-tui after launch.

## Acceptance Criteria
### AC-1 TUI bootstrap fails closed when runtime readiness is unavailable
Given `scripts/run/tau-unified.sh tui --bootstrap-runtime` starts or reuses a runtime,
when dashboard/runtime readiness does not become available within the bootstrap readiness window,
then the command exits nonzero with an actionable diagnostic instead of launching TUI.

### AC-2 Bind conflicts are caught before launching TUI
Given the configured `--bind` host:port is already occupied by an unmanaged process,
when `just tui` or `scripts/run/tau-unified.sh tui --bootstrap-runtime` attempts runtime bootstrap,
then the command exits nonzero and reports that the runtime is unavailable or the bind is not owned by the Tau launcher.

### AC-3 Healthy bootstrap behavior is preserved
Given the runtime starts successfully and readiness artifacts appear,
when `scripts/run/tau-unified.sh tui --bootstrap-runtime` runs,
then it launches the requested TUI mode with the existing timeout, retry, auth, and model arguments.

### AC-4 Runner-mode launcher tests remain deterministic
Given the launcher tests run with `TAU_UNIFIED_RUNNER`,
when bind-conflict and readiness failure regressions are exercised,
then the tests do not start the real Rust runtime and still prove the launcher command sequence.

## Conformance Cases
- C-01 / AC-1 / Regression:
  fake bootstrap leaves dashboard readiness artifacts absent, and `tui --bootstrap-runtime` fails before `runner_mode=tui` is logged.
- C-02 / AC-2 / Regression:
  a simulated unmanaged process occupies the target bind, and TUI launch refuses the bootstrap path with a bind/readiness diagnostic.
- C-03 / AC-3 / Functional:
  healthy runner bootstrap still logs `runner_mode=up` followed by `runner_mode=tui` and preserves timeout/retry flags.
- C-04 / AC-4 / Contract:
  `bash scripts/run/test-tau-unified.sh tui_bootstrap_readiness` runs only fake runner/fake readiness paths.

## Success Metrics / Observable Signals
- Operators see a clear pre-launch failure rather than an interactive shell attached to a missing or wrong runtime.
- The launcher has one explainable rule: TUI bootstrap must prove runtime readiness before launching TUI.
- Existing normal launcher tests continue to pass.
- No Cargo manifest or lockfile changes are required.

## Files To Touch
- `specs/3650/spec.md`
- `specs/3650/plan.md`
- `specs/3650/tasks.md`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `justfile` only if the `tui` recipe must pass an explicit launcher flag
