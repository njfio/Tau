# Spec: Issue #3646 - Fix tau-slack-runtime filter_next clippy blocker in send-file URL handling

Status: Implemented

## Objective
Remove the newly exposed `clippy::filter_next` failure in
`crates/tau-slack-runtime/src/slack_runtime.rs` so the package-scoped CI lane
for `#3631` can advance past the paired warning-debt blocker in the same run.

## Inputs/Outputs
Inputs:
- `crates/tau-slack-runtime/src/slack_runtime.rs`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- GitHub Actions failure evidence from PR `#3631`

Outputs:
- The `filter(...).next_back()` pattern in
  `send_file_source_filename_from_url` is replaced with the equivalent
  `rfind(...)` form.
- Package-scoped CI for `#3631` advances beyond the current `tau-slack-runtime`
  Clippy blocker.

## Boundaries / Non-goals
In scope:
- Mechanical `filter_next` cleanup in `tau-slack-runtime`
- Spec artifacts for `#3646`

Out of scope:
- Behavior changes to Slack file-upload semantics
- Broader refactors in `tau-slack-runtime`
- CI workflow changes unrelated to this concrete blocker

## Failure Modes
- GitHub Actions Clippy on Rust 1.94 flags `filter(...).next_back()` on a
  double-ended iterator as `filter_next` under `-D warnings`.
- The branch-level package-scoped validation lane reaches `tau-slack-runtime`
  only after the earlier blocker chain is fixed, so this lint now fails in the
  same run as `tau-onboarding`.

## Acceptance Criteria
### AC-1 tau-slack-runtime no longer uses filter-next-back at the failure site
Given the filename extraction helper in `tau-slack-runtime`,
when the Clippy cleanup is applied,
then the iterator logic uses `rfind(...)` instead of
`filter(...).next_back()`.

### AC-2 behavior remains unchanged
Given the existing send-file URL filename extraction semantics,
when the iterator logic is simplified,
then it still returns the last non-empty path segment and still falls back to
`attachment.bin` when no such segment exists.

### AC-3 branch validation advances beyond the tau-slack-runtime blocker
Given the exact reproduction command for `#3631`,
when `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
runs in CI,
then it advances beyond the current `tau-slack-runtime` `filter_next` failure.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `crates/tau-slack-runtime/src/slack_runtime.rs` uses `rfind(...)` at the
  current failure site in `send_file_source_filename_from_url`.
- C-02 / AC-2 / Review:
  The helper still chooses the last non-empty URL path segment and still falls
  back to `attachment.bin` for empty/missing path segments.
- C-03 / AC-3 / Regression:
  PR `#3631` no longer fails on the `tau-slack-runtime`
  `clippy::filter_next` error in GitHub Actions.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3646/spec.md`
- `specs/3646/plan.md`
- `specs/3646/tasks.md`
- `crates/tau-slack-runtime/src/slack_runtime.rs`

## Test Plan
- RED: capture the GitHub Actions failure from PR `#3631` showing the
  `tau-slack-runtime` `filter_next` error.
- GREEN: replace the iterator pattern and rerun
  `cargo clippy -p tau-slack-runtime --all-targets --all-features --no-deps -- -D warnings`.
- Regression: rerun
  `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
  locally and then on PR `#3631`.

## Success Metrics / Observable Signals
- `tau-slack-runtime` is no longer a hard failure in the package-scoped CI lane
  for `#3631`.
