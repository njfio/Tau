# Plan: Issue #3648 - Fix tau-browser-automation live fixture launcher race in CI

## Approach
1. Use the GitHub Actions failure on PR `#3631` as RED evidence.
2. Replace the unstable shell-script fixture in the failing
   `tau-browser-automation` integration test with the same stable temporary
   executable pattern already used by neighboring live-fixture tests.
3. Keep the runtime behavior and assertion contract unchanged: the executor must
   still fail in a way that maps to the retryable backend-unavailable result.
4. Rerun the focused `tau-browser-automation` selector and then rerun the exact
   `fast-validate --base <sha>` reproduction.

## Affected Areas
- `crates/tau-browser-automation/src/browser_automation_live.rs`
- `specs/milestones/m330/index.md`
- `specs/3648/`

## Compatibility / Contract Notes
- No production browser automation behavior changes are intended.
- This is a test-fixture stability fix only.

## Risks / Mitigations
- Risk: changing the temporary executable path could weaken the test’s intended
  failure contract.
  Mitigation: preserve the same public assertions and keep the executor failure
  mode focused on `execute-action`.
- Risk: the race could be broader than the shell-script fixture itself.
  Mitigation: use the neighboring Python-based helper path that is already
  stable in this test module; if the race persists, widen investigation only
  after rerunning the focused selector.
- Risk: another package-scoped blocker may surface immediately after this fix.
  Mitigation: keep the PR under observation after the push and peel the next
  blocker directly from GitHub logs.

## Verification
- `cargo test -p tau-browser-automation --lib integration_live_fixture_maps_executor_failures_to_retryable_backend_unavailable -- --nocapture`
- `cargo test -p tau-browser-automation --lib -- --nocapture`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- PR `#3631` GitHub Actions rerun
