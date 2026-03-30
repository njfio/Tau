# Spec: Issue #3639 - Contain tau-extensions deprecation lint in package-scoped clippy

Status: Implemented

## Objective
Keep the intentionally deprecated `tau-extensions` compatibility crate from
failing package-scoped clippy solely because its own internal implementation
still references deprecated public types during the staged migration to
`tau-skills`.

## Inputs/Outputs
Inputs:
- `crates/tau-extensions/src/lib.rs`
- `crates/tau-extensions/src/**`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- local `cargo clippy` failure evidence for `tau-extensions`

Outputs:
- `tau-extensions` no longer fails local or package-scoped clippy solely on
  deprecated-item usage inside the deprecated compatibility crate itself.
- The exact `#3631` fast-validate reproduction advances beyond the current
  `tau-extensions` blocker.

## Boundaries / Non-goals
In scope:
- A narrow lint-containment change for the deprecated `tau-extensions` crate
- Spec artifacts for `#3639`
- Scoped verification proving package-scoped clippy advances past this blocker

Out of scope:
- Migrating `tau-extensions` callers to `tau-skills`
- Rewriting extension manifest/runtime behavior
- Workspace-wide deprecation cleanup in unrelated crates

## Failure Modes
- `tau-extensions` marks its public compatibility types as deprecated, then
  immediately uses those same types internally, which becomes a hard failure
  under `-D warnings`.
- Package-scoped clippy on `#3631` reaches `tau-extensions` after the earlier
  compatibility-crate blockers are removed, so this deprecated crate now
  blocks unrelated branch validation.

## Acceptance Criteria
### AC-1 tau-extensions no longer fails on self-referential deprecation lint
Given `cargo clippy -p tau-extensions --all-targets --all-features --no-deps -- -D warnings`,
when the deprecated compatibility crate is linted,
then internal references to its own deprecated public items do not fail the
command.

### AC-2 the fix stays behavior-neutral
Given existing `tau-extensions` tests and runtime code,
when the deprecation-lint containment change is applied,
then no runtime behavior or public contract semantics are changed.

### AC-3 fast-validate advances beyond the tau-extensions blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb` runs,
then it advances beyond the prior `tau-extensions` deprecation failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `cargo clippy -p tau-extensions --all-targets --all-features --no-deps -- -D warnings`
  passes without deprecated-item errors from `tau-extensions`.
- C-02 / AC-2 / Regression:
  Existing targeted tests in `tau-extensions` still pass after the lint
  containment change.
- C-03 / AC-3 / Regression:
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  advances beyond the prior `tau-extensions` failure site.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3639/spec.md`
- `specs/3639/plan.md`
- `specs/3639/tasks.md`
- `crates/tau-extensions/src/lib.rs`

## Test Plan
- RED: capture `cargo clippy` failure output for `tau-extensions`.
- GREEN: apply the narrow lint-containment change and rerun scoped clippy.
- Regression: rerun the exact `fast-validate --base <sha>` reproduction and one
  targeted `tau-extensions` test selector.

## Success Metrics / Observable Signals
- `tau-extensions` is no longer the first hard failure in the `#3631`
  package-scoped clippy path.
- Any remaining `#3631` failure is further downstream than the deprecated
  compatibility crate boundary.
