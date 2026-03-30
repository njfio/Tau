# Spec: Issue #3636 - Scope fast-validate fmt checks to changed Rust surface

Status: Implemented

## Objective
Prevent `scripts/dev/fast-validate.sh --base <sha>` from failing narrow PRs on
unrelated workspace formatting drift by keeping fmt checks aligned with the
same PR-scoped Rust surface used for package-scoped clippy and test execution.

## Inputs/Outputs
Inputs:
- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`
- CI invocation from `.github/workflows/ci.yml`
- Failure evidence from PR `#3631`

Outputs:
- Scoped fmt behavior for PR validation when the diff stays package-scoped.
- Preserved full-workspace fmt behavior when full mode or workspace-wide scope
  is explicitly required.
- Regression coverage proving unrelated fmt drift no longer blocks narrow PRs.

## Boundaries / Non-goals
In scope:
- Fmt-check behavior in `scripts/dev/fast-validate.sh`
- Shell regression coverage in `scripts/dev/test-fast-validate.sh`
- Documentation/spec artifacts for `#3636`

Out of scope:
- Reformatting unrelated workspace files.
- Broad redesign of clippy/test package scoping.
- Changing GitHub Actions workflow structure beyond using the existing script.

## Failure Modes
- PR-scoped validation still runs `cargo fmt --all -- --check`.
- Existing fmt drift on unrelated files in `master` causes `Validate Rust scope`
  to fail even when the PR does not touch those files.
- Narrow blocker-fix PRs remain red after `#3632` because fmt scope is wider
  than clippy/test scope.

## Acceptance Criteria
### AC-1 PR-scoped fmt checks stay on changed Rust files
Given a pull-request style invocation with `--base <sha>` and a diff that does
not require full-workspace validation,
when `fast-validate.sh` runs fmt validation,
then it checks only the changed Rust source files in scope instead of the full
workspace.

### AC-2 Full-workspace modes keep full fmt enforcement
Given `--full` or a diff that explicitly expands to full-workspace scope,
when `fast-validate.sh` runs fmt validation,
then it still executes `cargo fmt --all -- --check`.

### AC-3 Regression coverage reproduces unrelated fmt drift
Given a fixture repo with an unrelated unformatted Rust file outside the PR
surface and a changed Rust file inside the PR surface,
when the regression test runs a PR-scoped fast-validate invocation,
then the run does not fail on the unrelated file.

### AC-4 Scoped fmt still catches changed-file formatting errors
Given a changed Rust file in the PR surface that is itself unformatted,
when the PR-scoped fmt check runs,
then the script still fails with a formatting diff for that changed file.

## Conformance Cases
- C-01 / AC-1 / Functional:
  PR-scoped validation runs rustfmt only on changed Rust files.
- C-02 / AC-2 / Functional:
  Full-workspace mode preserves `cargo fmt --all -- --check`.
- C-03 / AC-3 / Regression:
  A fixture with unrelated fmt drift outside the diff passes scoped fmt.
- C-04 / AC-4 / Regression:
  A changed unformatted Rust file in scope still fails scoped fmt.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3636/spec.md`
- `specs/3636/plan.md`
- `specs/3636/tasks.md`
- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`

## Test Plan
- RED: extend `scripts/dev/test-fast-validate.sh` with a fixture that fails
  under current whole-workspace fmt behavior because of unrelated drift.
- GREEN: update `fast-validate.sh` and rerun the script until the unrelated
  drift case passes while the in-scope unformatted case still fails.

## Success Metrics / Observable Signals
- `#3631` no longer fails in `Validate Rust scope` because of unrelated
  formatting drift outside its diff.
- `./scripts/dev/test-fast-validate.sh` covers both unrelated-drift pass and
  in-scope-drift fail behavior.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `fast-validate.sh --base 36dd1b5e...` now reports `changed_rust_files=2` and runs `rustfmt --edition 2021 --check` only on the changed `live_rl_runtime` files. |
| AC-2 | ✅ | `fast-validate.sh` still routes to `cargo fmt --all -- --check` when `--full` or full-workspace scope is active. |
| AC-3 | ✅ | `scripts/dev/test-fast-validate.sh` now builds a fixture where `crates/other/src/lib.rs` is unformatted in the base repo, and the scoped run passes because that file is outside the diff. |
| AC-4 | ✅ | `scripts/dev/test-fast-validate.sh` now builds a fixture where `crates/app/src/lib.rs` is the changed unformatted file, and the scoped run fails on that path. |

## Validation
- GREEN: `./scripts/dev/test-fast-validate.sh`
  - Result: `fast-validate scope tests passed`
- GREEN reproduction of the prior false positive:
  - `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  - Result: fmt now passes on only the changed Rust files and the run advances to package-scoped clippy instead of failing on unrelated workspace fmt drift.
