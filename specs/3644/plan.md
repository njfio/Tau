# Plan: Issue #3644 - Fix tau-gateway manual_unwrap_or clippy blocker in gateway config runtime

## Approach
1. Use the GitHub Actions failure on PR `#3631` as RED evidence.
2. Replace the handwritten `match` on
   `u64::try_from(state.config.runtime_heartbeat.interval.as_millis())` with
   the equivalent `unwrap_or(u64::MAX)` form.
3. Rerun local `tau-gateway` Clippy and then rerun the exact
   `fast-validate --base <sha>` reproduction.
4. Push the fix back to PR `#3631` and watch for the next concrete blocker, if
   any.

## Affected Areas
- `crates/tau-gateway/src/gateway_openresponses/config_runtime.rs`
- `specs/milestones/m330/index.md`
- `specs/3644/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- The simplified expression must preserve the existing fallback to `u64::MAX`
  when the conversion cannot fit in `u64`.

## Risks / Mitigations
- Risk: the local toolchain may not reproduce the exact CI lint.
  Mitigation: use the GitHub log as RED evidence, keep the change purely
  mechanical, and rerun the exact PR validation path after the edit.
- Risk: another package-scoped Clippy blocker may surface immediately after
  `tau-gateway` passes.
  Mitigation: keep the PR under observation after the push and peel the next
  blocker directly from GitHub logs.

## Verification
- `cargo clippy -p tau-gateway --all-targets --all-features --no-deps -- -D warnings`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- PR `#3631` GitHub Actions rerun
