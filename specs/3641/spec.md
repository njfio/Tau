# Spec: Issue #3641 - Fix tau-onboarding clippy derivable_impls blocker

Status: Implemented

## Objective
Remove the current `clippy::derivable_impls` failure in
`tau-onboarding::SelfImprovementConfig` so package-scoped validation on `#3631`
can continue past this real code-quality blocker.

## Inputs/Outputs
Inputs:
- `crates/tau-onboarding/src/config_file.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- local `cargo clippy` failure evidence for `tau-onboarding`

Outputs:
- `SelfImprovementConfig` uses derive-based `Default`.
- `tau-onboarding` no longer fails the current `derivable_impls` lint.
- The exact `#3631` fast-validate reproduction advances beyond the current
  `tau-onboarding` blocker.

## Boundaries / Non-goals
In scope:
- Mechanical `Default`-derive cleanup for `SelfImprovementConfig`
- Spec artifacts for `#3641`
- Scoped verification proving package-scoped clippy advances past this blocker

Out of scope:
- Behavior changes to onboarding config semantics
- Broader cleanup in unrelated onboarding config structs

## Failure Modes
- `SelfImprovementConfig` manually implements `Default` even though all fields
  already use the language default values, so clippy flags the impl as
  derivable.

## Acceptance Criteria
### AC-1 tau-onboarding no longer fails the derivable_impls lint
Given `cargo clippy -p tau-onboarding --all-targets --all-features --no-deps -- -D warnings`,
when `tau-onboarding` is linted,
then `SelfImprovementConfig` no longer fails `clippy::derivable_impls`.

### AC-2 behavior remains unchanged
Given the existing onboarding config semantics,
when `SelfImprovementConfig` moves from a manual `Default` impl to a derived
default,
then the resulting default field values remain all `false`.

### AC-3 fast-validate advances beyond the tau-onboarding blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb` runs,
then it advances beyond the prior `tau-onboarding` failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `cargo clippy -p tau-onboarding --all-targets --all-features --no-deps -- -D warnings`
  passes without the `derivable_impls` error.
- C-02 / AC-2 / Review:
  The derived `Default` for `SelfImprovementConfig` still yields `false` for
  every field.
- C-03 / AC-3 / Regression:
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  advances beyond the prior `tau-onboarding` failure site.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3641/spec.md`
- `specs/3641/plan.md`
- `specs/3641/tasks.md`
- `crates/tau-onboarding/src/config_file.rs`

## Test Plan
- RED: capture `cargo clippy` failure output for `tau-onboarding`.
- GREEN: replace the manual `Default` impl with derive-based `Default` and
  rerun scoped clippy.
- Regression: rerun the exact `fast-validate --base <sha>` reproduction.

## Success Metrics / Observable Signals
- `tau-onboarding` is no longer the first hard failure in the `#3631`
  package-scoped clippy path.
- Any remaining `#3631` failure is further downstream than the onboarding
  config lint.
