# Spec: Issue #3632 - Make fast-validate resilient to shallow PR history

Status: Implemented

## Objective
Prevent `scripts/dev/fast-validate.sh --base <sha>` from silently collapsing
to zero changed files when CI has the base commit object but not enough local
history to compute a merge base via `base...HEAD`.

## Inputs/Outputs
Inputs:
- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`
- Shallow-checkout CI behavior from `.github/workflows/ci.yml`

Outputs:
- Scoped changed-file detection that still works for master-based PRs with
  shallow history.
- Deterministic regression coverage reproducing the missing-merge-base case.
- Bounded diagnostic output when triple-dot diff cannot be computed.

## Boundaries / Non-goals
In scope:
- Validation scoping logic in `scripts/dev/fast-validate.sh`
- Script regression tests in `scripts/dev/test-fast-validate.sh`
- Documentation/spec artifacts for `#3632`

Out of scope:
- Changing the CI workflow checkout depth.
- Broad validation strategy redesign.
- Fixing unrelated fmt/clippy debt exposed by full-workspace fallback.

## Failure Modes
- `git diff ${base_ref}...HEAD` emits `fatal: ... no merge base`.
- The script swallows that fatal and records zero changed files.
- Zero changed files cause the script to report no impacted packages and drift
  into full-workspace validation.
- CI then fails on unrelated workspace-wide drift instead of scoped PR drift.

## Acceptance Criteria
### AC-1 Shallow-history missing-merge-base case is reproduced in tests
Given a shallow clone that has `HEAD` and a fetched base commit but not the
intermediate ancestry needed for `base...HEAD`,
when the fast-validate regression test runs,
then it reproduces the pre-fix failure mode deterministically.

### AC-2 fast-validate preserves scoped changed-file detection
Given `--base <sha>` with a base commit object available but no local merge
base,
when `fast-validate.sh` computes changed files,
then it uses a bounded fallback that still returns the actual files changed
between the base commit and `HEAD`.

### AC-3 The script emits bounded diagnostics instead of silent collapse
Given the missing-merge-base case,
when fallback behavior is used,
then the script avoids raw git fatal noise and emits an explicit warning that
describes the fallback path.

### AC-4 Regression coverage proves scoped validation no longer expands
Given the deterministic shallow-history fixture,
when the script runs in check-only mode with fmt skipped,
then output shows nonzero changed files and avoids the prior zero-file collapse.

## Conformance Cases
- C-01 / AC-1 / Regression:
  The test harness creates a shallow clone with fetched base commit and missing
  intermediate history.
- C-02 / AC-2 / Functional:
  The script falls back from `base...HEAD` to `base..HEAD` when the merge base
  is unavailable locally.
- C-03 / AC-3 / Functional:
  Output includes an explicit warning describing the two-dot diff fallback.
- C-04 / AC-4 / Regression:
  Test output includes `changed_files=1` (or another nonzero count) and does
  not include the prior `fatal: ... no merge base` leak.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3632/spec.md`
- `specs/3632/plan.md`
- `specs/3632/tasks.md`
- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`

## Test Plan
- RED: run `./scripts/dev/test-fast-validate.sh` after adding a shallow-history
  regression assertion that fails on current behavior.
- GREEN: rerun `./scripts/dev/test-fast-validate.sh` after the fallback fix.
- Focused regression: run the same shallow-history case and verify changed-file
  output is nonzero with bounded warning output.

## Success Metrics / Observable Signals
- The shallow-history test proves the failure mode and then passes after the
  fix.
- `fast-validate` no longer collapses to zero changed files in the reproduced
  missing-merge-base case.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `scripts/dev/test-fast-validate.sh` now builds a temp shallow clone with a fetched base commit and missing intermediate history. |
| AC-2 | ✅ | `collect_changed_files()` now falls back from `base...HEAD` to `base..HEAD` when no local merge base is available. |
| AC-3 | ✅ | The reproduced output now emits `warning: base ref '<sha>' has no local merge base with HEAD; using two-dot diff fallback` instead of raw git fatal noise. |
| AC-4 | ✅ | The reproduced shallow-history run now reports `changed_files=1 impacted_packages=0` and stays scoped. |

## Validation
- RED fixture output before the fix:
  - `fatal: <base>...HEAD: no merge base`
  - `changed_files=0 impacted_packages=0`
- GREEN: `./scripts/dev/test-fast-validate.sh`
  - Result: `fast-validate scope tests passed`
- GREEN shallow-history spot check:
  - Output contains bounded warning
  - Output contains `changed_files=1`
  - Output does not contain `fatal:`
