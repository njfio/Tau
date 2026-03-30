# Spec: Issue #3643 - Fix tau-memory clippy derivable_impls blockers in memory runtime

Status: Implemented

## Objective
Remove the newly exposed `clippy::derivable_impls` failures in
`crates/tau-memory/src/runtime.rs` so the package-scoped CI lane for `#3631`
can advance past the next real warning-debt blocker.

## Inputs/Outputs
Inputs:
- `crates/tau-memory/src/runtime.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- GitHub Actions failure evidence from PR `#3631`

Outputs:
- `MemoryRelationType` uses a derive-based `Default` with an explicit default
  variant.
- `MemoryType` uses a derive-based `Default` with an explicit default variant.
- Package-scoped CI for `#3631` advances beyond the current `tau-memory`
  clippy blocker.

## Boundaries / Non-goals
In scope:
- Mechanical `Default`-derive cleanup for `MemoryRelationType`
- Mechanical `Default`-derive cleanup for `MemoryType`
- Spec artifacts for `#3643`

Out of scope:
- Behavior changes to relation parsing, memory typing, or importance defaults
- Broader `tau-memory` refactors
- CI workflow changes unrelated to this concrete blocker

## Failure Modes
- GitHub Actions Clippy on Rust 1.94 flags the manual `Default` impls for
  `MemoryRelationType` and `MemoryType` as derivable.
- The branch-level package-scoped validation lane reaches `tau-memory` only
  after the earlier blocker chain is fixed, so this lint is now the next hard
  stop on `#3631`.

## Acceptance Criteria
### AC-1 tau-memory no longer uses manual defaults for derivable enums
Given `MemoryRelationType` and `MemoryType` in `tau-memory`,
when the clippy cleanup is applied,
then both enums derive `Default` and mark the current default variant
explicitly.

### AC-2 behavior remains unchanged
Given the existing `tau-memory` runtime semantics,
when the manual enum defaults are removed,
then the default relation remains `RelatedTo` and the default memory type
remains `Observation`.

### AC-3 branch validation advances beyond the tau-memory blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
runs in CI,
then it advances beyond the current `tau-memory` clippy failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `crates/tau-memory/src/runtime.rs` derives `Default` on both enums and uses
  `#[default]` on `RelatedTo` and `Observation`.
- C-02 / AC-2 / Review:
  The effective default variants remain unchanged after the mechanical cleanup.
- C-03 / AC-3 / Regression:
  PR `#3631` no longer fails on the `tau-memory` `derivable_impls` errors in
  GitHub Actions.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3643/spec.md`
- `specs/3643/plan.md`
- `specs/3643/tasks.md`
- `crates/tau-memory/src/runtime.rs`

## Test Plan
- RED: capture the GitHub Actions failure from PR `#3631` showing the
  `tau-memory` `derivable_impls` errors.
- GREEN: replace the manual enum defaults with derive-based defaults and rerun
  `cargo clippy -p tau-memory --all-targets --all-features --no-deps -- -D warnings`.
- Regression: rerun `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  locally and then on PR `#3631`.

## Success Metrics / Observable Signals
- `tau-memory` is no longer the first hard failure in the package-scoped CI
  lane for `#3631`.
