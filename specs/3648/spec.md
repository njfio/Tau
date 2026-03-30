# Spec: Issue #3648 - Fix tau-browser-automation live fixture launcher race in CI

Status: Implemented

## Objective
Remove the newly exposed `tau-browser-automation` CI-only test race where the
live browser automation fixture launches a temporary shell script and
intermittently fails on Linux with `Text file busy (os error 26)` before the
intended retryable backend-unavailable mapping path is exercised.

## Inputs/Outputs
Inputs:
- `crates/tau-browser-automation/src/browser_automation_live.rs`
- GitHub Actions failure evidence from PR `#3631`
- Existing local live-fixture helpers in the same test module

Outputs:
- The failing live-fixture integration test uses a deterministic temporary
  executable pattern that is stable on Linux CI.
- The executor failure still maps to
  `browser_automation_backend_unavailable` with `503`.
- Package-scoped validation on PR `#3631` advances beyond the
  `Text file busy` failure in `tau-browser-automation`.

## Boundaries / Non-goals
In scope:
- Test-fixture stabilization in `tau-browser-automation`
- Spec artifacts for `#3648`

Out of scope:
- Production behavior changes to browser automation runtime code
- Refactoring unrelated browser automation tests
- CI workflow changes unrelated to this fixture race

## Failure Modes
- PR `#3631` now fails in
  `browser_automation_live::tests::integration_live_fixture_maps_executor_failures_to_retryable_backend_unavailable`
  after the earlier validation blockers were removed.
- The failing test writes a temporary shell script
  `failing-playwright-cli.sh`, marks it executable, and launches it
  immediately through `PlaywrightCliActionExecutor`.
- On the Linux GitHub runner, the launch intermittently fails with
  `Text file busy (os error 26)`, so the test never reaches the intended error
  mapping assertion.
- Neighboring live-fixture tests already use a Python-based temporary mock
  executable and do not exhibit this launcher race.

## Acceptance Criteria
### AC-1 The failing live-fixture test no longer relies on the unstable shell-script launcher path
Given the retryable backend-unavailable integration test in
`browser_automation_live.rs`,
when it provisions its temporary executor fixture,
then it uses a stable executable pattern already proven by neighboring tests
instead of the ad hoc shell script that triggers `Text file busy` in CI.

### AC-2 The behavioral contract remains unchanged
Given the same failing-executor scenario,
when the updated test runs,
then `run_browser_automation_live_fixture(...)` still returns a summary with
one retryable failure and the failure maps to
`browser_automation_backend_unavailable` / HTTP `503`.

### AC-3 Package-scoped CI advances beyond the launcher race
Given the package-scoped validation path for PR `#3631`,
when `tau-browser-automation` tests run under CI,
then they no longer fail on `failed to launch browser automation executor` with
`Text file busy (os error 26)`.

## Conformance Cases
- C-01 / AC-1 / Functional:
  The retryable backend-unavailable integration test no longer writes or
  launches `failing-playwright-cli.sh`; it uses the stable mock-executable
  helper pattern.
- C-02 / AC-2 / Regression:
  `cargo test -p tau-browser-automation --lib integration_live_fixture_maps_executor_failures_to_retryable_backend_unavailable -- --nocapture`
  passes and still asserts one retryable failure.
- C-03 / AC-3 / Regression:
  PR `#3631` no longer fails on the `Text file busy (os error 26)` launcher
  race in `tau-browser-automation`.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3648/spec.md`
- `specs/3648/plan.md`
- `specs/3648/tasks.md`
- `crates/tau-browser-automation/src/browser_automation_live.rs`

## Test Plan
- RED: use the GitHub Actions failure from PR `#3631` showing the
  `Text file busy (os error 26)` launch error in
  `integration_live_fixture_maps_executor_failures_to_retryable_backend_unavailable`.
- GREEN: update the fixture helper path and rerun
  `cargo test -p tau-browser-automation --lib integration_live_fixture_maps_executor_failures_to_retryable_backend_unavailable -- --nocapture`.
- Regression: rerun
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  locally and then on PR `#3631`.

## Success Metrics / Observable Signals
- `tau-browser-automation` advances beyond the current Linux launcher race on
  PR `#3631`.
- The next PR failure, if any, is a different concrete blocker.
