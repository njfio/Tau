# Spec: Issue #3645 - Fix tau-onboarding cmp_owned clippy blocker in startup transport modes

Status: Implemented

## Objective
Remove the newly exposed `clippy::cmp_owned` failure in
`crates/tau-onboarding/src/startup_transport_modes.rs` so the package-scoped CI
lane for `#3631` can advance past the next real warning-debt blocker.

## Inputs/Outputs
Inputs:
- `crates/tau-onboarding/src/startup_transport_modes.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- GitHub Actions failure evidence from PR `#3631`

Outputs:
- The owned comparison against `PathBuf::from(".tau/runtime-heartbeat/state.json")`
  is replaced with the equivalent borrowed-path comparison.
- Package-scoped CI for `#3631` advances beyond the current `tau-onboarding`
  Clippy blocker.

## Boundaries / Non-goals
In scope:
- Mechanical `cmp_owned` cleanup in `tau-onboarding`
- Spec artifacts for `#3645`

Out of scope:
- Behavior changes to onboarding startup transport modes
- Broader refactors in `tau-onboarding`
- CI workflow changes unrelated to this concrete blocker

## Failure Modes
- GitHub Actions Clippy on Rust 1.94 flags the comparison between
  `runtime_heartbeat.state_path` and `PathBuf::from(".tau/runtime-heartbeat/state.json")`
  as `cmp_owned` under `-D warnings`.
- The branch-level package-scoped validation lane reaches `tau-onboarding`
  only after the earlier blocker chain is fixed, so this lint is now a hard
  stop on `#3631`.

## Acceptance Criteria
### AC-1 tau-onboarding no longer compares against an owned temporary PathBuf
Given the runtime heartbeat path override in `tau-onboarding`,
when the Clippy cleanup is applied,
then the comparison uses the borrowed path literal form instead of constructing
`PathBuf::from(...)` only for equality.

### AC-2 behavior remains unchanged
Given the existing onboarding runtime heartbeat semantics,
when the comparison is simplified,
then the default state path still remaps into `cli.gateway_state_dir` and
non-default paths remain untouched.

### AC-3 branch validation advances beyond the tau-onboarding blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
runs in CI,
then it advances beyond the current `tau-onboarding` `cmp_owned` failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `crates/tau-onboarding/src/startup_transport_modes.rs` compares
  `runtime_heartbeat.state_path` against the borrowed path literal instead of
  `PathBuf::from(...)` at the current failure site.
- C-02 / AC-2 / Review:
  The remap still occurs only for the default runtime heartbeat state file and
  still rewrites it into `cli.gateway_state_dir`.
- C-03 / AC-3 / Regression:
  PR `#3631` no longer fails on the `tau-onboarding` `clippy::cmp_owned`
  error in GitHub Actions.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3645/spec.md`
- `specs/3645/plan.md`
- `specs/3645/tasks.md`
- `crates/tau-onboarding/src/startup_transport_modes.rs`

## Test Plan
- RED: capture the GitHub Actions failure from PR `#3631` showing the
  `tau-onboarding` `cmp_owned` error.
- GREEN: replace the owned comparison and rerun
  `cargo clippy -p tau-onboarding --all-targets --all-features --no-deps -- -D warnings`.
- Regression: rerun
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  locally and then on PR `#3631`.

## Success Metrics / Observable Signals
- `tau-onboarding` is no longer the first hard failure in the package-scoped CI
  lane for `#3631`.
