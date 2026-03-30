# Spec: Issue #3644 - Fix tau-gateway manual_unwrap_or clippy blocker in gateway config runtime

Status: Implemented

## Objective
Remove the newly exposed `clippy::manual_unwrap_or` failure in
`crates/tau-gateway/src/gateway_openresponses/config_runtime.rs` so the
package-scoped CI lane for `#3631` can advance past the next real warning-debt
blocker.

## Inputs/Outputs
Inputs:
- `crates/tau-gateway/src/gateway_openresponses/config_runtime.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- GitHub Actions failure evidence from PR `#3631`

Outputs:
- The manual `match` on `u64::try_from(...as_millis())` is replaced with the
  equivalent `unwrap_or(u64::MAX)` form.
- Package-scoped CI for `#3631` advances beyond the current `tau-gateway`
  Clippy blocker.

## Boundaries / Non-goals
In scope:
- Mechanical `manual_unwrap_or` cleanup in `tau-gateway`
- Spec artifacts for `#3644`

Out of scope:
- Behavior changes to gateway config responses
- Broader refactors in `tau-gateway`
- CI workflow changes unrelated to this concrete blocker

## Failure Modes
- GitHub Actions Clippy on Rust 1.94 flags the manual `match` converting the
  runtime heartbeat interval millis into `u64` as a `manual_unwrap_or`
  pattern.
- The branch-level package-scoped validation lane reaches `tau-gateway` only
  after the earlier blocker chain is fixed, so this lint is now the next hard
  stop on `#3631`.

## Acceptance Criteria
### AC-1 tau-gateway no longer uses the manual unwrap-or pattern
Given `handle_gateway_config_get` in `tau-gateway`,
when the Clippy cleanup is applied,
then the `u64::try_from(...as_millis())` conversion uses `unwrap_or(u64::MAX)`
instead of a handwritten `match`.

### AC-2 behavior remains unchanged
Given the existing gateway config semantics,
when the manual pattern is simplified,
then successful conversions still return the converted value and overflow still
falls back to `u64::MAX`.

### AC-3 branch validation advances beyond the tau-gateway blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
runs in CI,
then it advances beyond the current `tau-gateway` Clippy failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `crates/tau-gateway/src/gateway_openresponses/config_runtime.rs` uses the
  `unwrap_or(u64::MAX)` form at the current failure site.
- C-02 / AC-2 / Review:
  The simplified expression still returns the same value on success and the
  same `u64::MAX` fallback on conversion failure.
- C-03 / AC-3 / Regression:
  PR `#3631` no longer fails on the `tau-gateway`
  `clippy::manual_unwrap_or` error in GitHub Actions.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3644/spec.md`
- `specs/3644/plan.md`
- `specs/3644/tasks.md`
- `crates/tau-gateway/src/gateway_openresponses/config_runtime.rs`

## Test Plan
- RED: capture the GitHub Actions failure from PR `#3631` showing the
  `tau-gateway` `manual_unwrap_or` error.
- GREEN: replace the manual pattern and rerun
  `cargo clippy -p tau-gateway --all-targets --all-features --no-deps -- -D warnings`.
- Regression: rerun `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  locally and then on PR `#3631`.

## Success Metrics / Observable Signals
- `tau-gateway` is no longer the first hard failure in the package-scoped CI
  lane for `#3631`.
