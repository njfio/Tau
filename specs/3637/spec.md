# Spec: Issue #3637 - Resolve tau-coding-agent warning debt exposed by package-scoped clippy

Status: Implemented

## Objective
Clear the remaining `-D warnings` failures that block PR `#3631` after the CI
scope defects were removed, starting with the package-scoped clippy path that
now reaches real warning debt in `tau-safety` and `tau-coding-agent`.

## Inputs/Outputs
Inputs:
- `crates/tau-safety/src/lib.rs`
- `crates/tau-coding-agent/src/**`
- `scripts/dev/fast-validate.sh` reproduction path on `#3631`
- CI failure evidence from PR `#3631`

Outputs:
- `tau-safety` no longer fails clippy on `derivable_impls`.
- `tau-coding-agent` no longer fails package-scoped clippy because of known
  deprecated compatibility surfaces or dead-code in the self-modification
  runtime.
- Scoped validation for `#3631` advances past the current warning-debt blocker.

## Boundaries / Non-goals
In scope:
- Narrow warning fixes in `tau-safety`
- Narrow warning fixes or justified targeted allows in `tau-coding-agent`
- Spec artifacts for `#3637`

Out of scope:
- Broad migration away from `tau-extensions` or `tau-custom-command`
- Rewriting the self-modification runtime feature
- Workspace-wide lint cleanup outside the impacted validation path

## Failure Modes
- `clippy::derivable_impls` fails in `tau-safety::SafetyMode`.
- Deprecated compatibility types from `tau_extensions` are intentionally still
  referenced by `tau-coding-agent`, but `-D warnings` treats those references
  as hard failures.
- Unused self-modification runtime types/functions fail as dead code even
  though the feature surface is intentionally staged and not yet wired.

## Acceptance Criteria
### AC-1 tau-safety clears the current clippy lint
Given package-scoped clippy validation on the `#3631` branch,
when `tau-safety` is linted,
then `SafetyMode` no longer fails `clippy::derivable_impls`.

### AC-2 tau-coding-agent compatibility surfaces no longer fail deprecation lint
Given package-scoped clippy validation on the `#3631` branch,
when `tau-coding-agent` is linted,
then intentional compatibility usage of deprecated `tau_extensions` types is
handled without turning the branch red.

### AC-3 staged self-modification runtime code no longer fails dead-code lint
Given package-scoped clippy validation on the `#3631` branch,
when `tau-coding-agent` is linted,
then the currently staged but intentionally unwired self-modification runtime
items no longer fail `dead_code`.

### AC-4 scoped validation advances beyond the current blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base <sha>` runs,
then it gets past the previously failing warning sites addressed in this task.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `cargo clippy` on the impacted path no longer reports `derivable_impls` for
  `crates/tau-safety/src/lib.rs`.
- C-02 / AC-2 / Functional:
  `cargo clippy -p tau-coding-agent --all-targets --all-features --no-deps -- -D warnings`
  no longer fails on deprecated `tau_extensions` usage in `tau-coding-agent`.
- C-03 / AC-3 / Functional:
  The same clippy command no longer fails on dead-code in
  `self_modification_runtime.rs`.
- C-04 / AC-4 / Regression:
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  advances beyond the current warning sites.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3637/spec.md`
- `specs/3637/plan.md`
- `specs/3637/tasks.md`
- `crates/tau-safety/src/lib.rs`
- `crates/tau-coding-agent/src/**`

## Test Plan
- RED: capture the current `cargo clippy` failures on `tau-safety` and
  `tau-coding-agent`.
- GREEN: rerun scoped clippy after each narrow fix until the targeted command
  passes.
- Regression: rerun the exact `fast-validate --base <sha>` reproduction from
  `#3631`.

## Success Metrics / Observable Signals
- `#3631` no longer fails on the current `tau-safety` and `tau-coding-agent`
  warning sites.
- The remaining CI surface, if any, is further downstream than the current
  warning-debt blocker.
