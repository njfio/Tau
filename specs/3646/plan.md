# Plan: Issue #3646 - Fix tau-slack-runtime filter_next clippy blocker in send-file URL handling

## Approach
1. Use the GitHub Actions failure on PR `#3631` as RED evidence.
2. Replace the `filter(...).next_back()` pattern with the equivalent
   `rfind(...)` form Clippy expects.
3. Rerun local `tau-slack-runtime` Clippy and then rerun the exact
   `fast-validate --base <sha>` reproduction.
4. Push the fix back to PR `#3631` and watch for the next concrete blocker, if
   any.

## Affected Areas
- `crates/tau-slack-runtime/src/slack_runtime.rs`
- `specs/milestones/m330/index.md`
- `specs/3646/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- The helper must still choose the last non-empty path segment and preserve the
  existing fallback filename.

## Risks / Mitigations
- Risk: the local toolchain may not reproduce the exact CI lint.
  Mitigation: use the GitHub log as RED evidence, keep the change purely
  mechanical, and rerun the exact PR validation path after the edit.
- Risk: another package-scoped Clippy blocker may surface immediately after
  `tau-slack-runtime` passes.
  Mitigation: keep the PR under observation after the push and peel the next
  blocker directly from GitHub logs.

## Verification
- `cargo clippy -p tau-slack-runtime --all-targets --all-features --no-deps -- -D warnings`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- PR `#3631` GitHub Actions rerun
