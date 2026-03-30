# Plan: Issue #3645 - Fix tau-onboarding cmp_owned clippy blocker in startup transport modes

## Approach
1. Use the GitHub Actions failure on PR `#3631` as RED evidence.
2. Replace the equality comparison against `PathBuf::from(...)` with the
   borrowed path literal form Clippy expects.
3. Rerun local `tau-onboarding` Clippy and then rerun the exact
   `fast-validate --base <sha>` reproduction.
4. Push the fix back to PR `#3631` and watch for the next concrete blocker, if
   any.

## Affected Areas
- `crates/tau-onboarding/src/startup_transport_modes.rs`
- `specs/milestones/m330/index.md`
- `specs/3645/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- The default heartbeat state-path remap must remain identical after the
  comparison is simplified.

## Risks / Mitigations
- Risk: the local toolchain may not reproduce the exact CI lint.
  Mitigation: use the GitHub log as RED evidence, keep the change purely
  mechanical, and rerun the exact PR validation path after the edit.
- Risk: another package-scoped Clippy blocker may surface immediately after
  `tau-onboarding` passes.
  Mitigation: keep the PR under observation after the push and peel the next
  blocker directly from GitHub logs.

## Verification
- `cargo clippy -p tau-onboarding --all-targets --all-features --no-deps -- -D warnings`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- PR `#3631` GitHub Actions rerun
