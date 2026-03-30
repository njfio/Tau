# Spec: Issue #3647 - Fix tau-coding-agent cli integration training store lock race in auth provider tests

Status: Implemented

## Objective
Remove the newly exposed `tau-coding-agent` CLI integration race where
subprocess-based tests initialize the default live RL SQLite training store in
parallel and intermittently fail with `database is locked` in GitHub Actions.

## Inputs/Outputs
Inputs:
- `crates/tau-coding-agent/tests/cli_integration.rs`
- `crates/tau-coding-agent/tests/cli_integration/auth_provider.rs`
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- GitHub Actions failure evidence from PR `#3631`

Outputs:
- The shared CLI integration command helper disables live RL runtime startup by
  default for subprocess-based CLI tests.
- Auth-provider CLI integration coverage remains intact while no longer touching
  `.tau/training/store.sqlite` during parallel test execution.
- Package-scoped validation on PR `#3631` advances beyond the current
  `database is locked` failure in `tau-coding-agent --test cli_integration`.

## Boundaries / Non-goals
In scope:
- CLI integration test harness isolation from live RL startup
- Spec artifacts for `#3647`

Out of scope:
- Production behavior changes to live RL runtime defaults
- Refactoring live RL runtime initialization in shipping code
- Adding new environment flags or changing the public live RL contract

## Failure Modes
- GitHub Actions runs `tau-coding-agent --test cli_integration` with parallel
  subprocess-based tests that share the default live RL training store path at
  `.tau/training/store.sqlite`.
- `LiveRlRuntimeConfig::from_env_map` enables live RL by default when
  `TAU_LIVE_RL_ENABLED` is unset, so CLI integration subprocesses initialize the
  same SQLite store even when the tests are not exercising live RL behavior.
- The auth-provider integration tests intermittently fail with
  `failed to initialize live RL training store ... database is locked`, which
  blocks PR `#3631` from reaching the actual oversized-file story.

## Acceptance Criteria
### AC-1 CLI integration helper disables live RL by default
Given the shared `binary_command()` helper for subprocess-based CLI integration
tests,
when it constructs a `tau-coding-agent` command,
then it sets `TAU_LIVE_RL_ENABLED=0` so live RL runtime initialization does not
run implicitly during unrelated CLI integration coverage.

### AC-2 auth-provider coverage remains behaviorally equivalent
Given the auth-provider CLI integration tests,
when they run through the updated helper,
then they still verify provider routing and fallback behavior without requiring
the live RL runtime or its SQLite training store.

### AC-3 package-scoped validation advances beyond the lock race
Given the package-scoped validation path for PR `#3631`,
when `tau-coding-agent --test cli_integration` runs under CI,
then it no longer fails on `failed to initialize live RL training store` /
`database is locked` for the auth-provider tests.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `binary_command()` in `crates/tau-coding-agent/tests/cli_integration.rs`
  exports `TAU_LIVE_RL_ENABLED=0`.
- C-02 / AC-2 / Regression:
  `cargo test -p tau-coding-agent --test cli_integration auth_provider:: -- --test-threads 2`
  passes with the updated helper.
- C-03 / AC-3 / Regression:
  PR `#3631` no longer fails on the `database is locked` live RL store
  initialization error in `tau-coding-agent --test cli_integration`.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3647/spec.md`
- `specs/3647/plan.md`
- `specs/3647/tasks.md`
- `crates/tau-coding-agent/tests/cli_integration.rs`

## Test Plan
- RED: use the GitHub Actions failure from PR `#3631` showing
  `database is locked` while initializing `.tau/training/store.sqlite`.
- GREEN: update the shared CLI integration helper and rerun
  `cargo test -p tau-coding-agent --test cli_integration auth_provider:: -- --test-threads 2`.
- Regression: rerun
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  locally and then on PR `#3631`.

## Success Metrics / Observable Signals
- `tau-coding-agent --test cli_integration` advances beyond the live RL SQLite
  lock race on PR `#3631`.
- The next PR failure, if any, is a different concrete blocker.
