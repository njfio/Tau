# Spec: Issue #3640 - Contain tau-tools extension compatibility deprecation lint

Status: Implemented

## Objective
Keep `tau-tools` lint-clean in package-scoped clippy while it still carries a
small number of intentional bridge points into the deprecated
`tau-extensions` surface during the staged migration to `tau-skills`.

## Inputs/Outputs
Inputs:
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/bash_tool.rs`
- `crates/tau-tools/src/tools/tests.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- local `cargo clippy` failure evidence for `tau-tools`

Outputs:
- `tau-tools` no longer fails local or package-scoped clippy solely on the
  intentional extension-compatibility bridge points.
- The exact `#3631` fast-validate reproduction advances beyond the current
  `tau-tools` blocker.

## Boundaries / Non-goals
In scope:
- Narrow deprecation-lint containment at the `tau-tools` bridge points that
  still consume `tau-extensions`
- Spec artifacts for `#3640`
- Scoped verification proving package-scoped clippy advances past this blocker

Out of scope:
- Migrating `tau-tools` fully off `tau-extensions`
- Broad crate-wide `allow(deprecated)` in `tau-tools`
- Behavior changes to tool registration or extension policy override flows

## Failure Modes
- `tau-tools` still imports and consumes deprecated extension registration and
  policy-override types as part of the compatibility bridge, and those
  intentional references become hard failures under `-D warnings`.
- Package-scoped clippy on `#3631` now reaches `tau-tools` after the earlier
  compatibility-crate blockers are removed, so the bridge points become the
  next blocker.

## Acceptance Criteria
### AC-1 tau-tools bridge points no longer fail deprecation lint
Given `cargo clippy -p tau-tools --all-targets --all-features --no-deps -- -D warnings`,
when `tau-tools` is linted,
then the intentional `tau-extensions` compatibility bridge points no longer
fail the command.

### AC-2 lint containment stays narrow
Given the `tau-tools` compatibility bridge code,
when deprecation warnings are contained,
then the containment is limited to the bridge imports/items/tests that still
depend on `tau-extensions`, rather than muting deprecation warnings across the
entire crate.

### AC-3 fast-validate advances beyond the tau-tools blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb` runs,
then it advances beyond the prior `tau-tools` deprecation failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `cargo clippy -p tau-tools --all-targets --all-features --no-deps -- -D warnings`
  passes without deprecated-item errors from the known extension bridge sites.
- C-02 / AC-2 / Review:
  The applied `allow(deprecated)` scope is limited to the bridge imports,
  functions, structs, and tests that still depend on `tau-extensions`.
- C-03 / AC-3 / Regression:
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  advances beyond the prior `tau-tools` failure site.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3640/spec.md`
- `specs/3640/plan.md`
- `specs/3640/tasks.md`
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/bash_tool.rs`
- `crates/tau-tools/src/tools/tests.rs`

## Test Plan
- RED: capture `cargo clippy` failure output for `tau-tools`.
- GREEN: apply narrow bridge-point allowances and rerun scoped clippy.
- Regression: rerun the exact `fast-validate --base <sha>` reproduction and one
  targeted `tau-tools` test selector.

## Success Metrics / Observable Signals
- `tau-tools` is no longer the first hard failure in the `#3631`
  package-scoped clippy path.
- Any remaining `#3631` failure is further downstream than the current
  extension bridge points in `tau-tools`.
