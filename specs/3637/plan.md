# Plan: Issue #3637 - Resolve tau-coding-agent warning debt exposed by package-scoped clippy

## Approach
1. Reproduce the current failures with two commands:
   - `cargo clippy -p tau-coding-agent --all-targets --all-features --no-deps -- -D warnings`
   - `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
2. Fix `tau-safety` first because CI currently reports that crate as the first
   hard error and the fix should be mechanical.
3. Fix `tau-coding-agent` second by:
   - containing intentional compatibility usage of deprecated
     `tau_extensions` types with narrow, explicit allowances rather than broad
     crate-wide suppression,
   - containing staged self-modification-runtime dead-code with explicit,
     local justification.
4. Rerun the scoped clippy command and the `fast-validate` reproduction to
   verify the branch advances past this blocker.

## Affected Areas
- `crates/tau-safety/src/lib.rs`
- `crates/tau-coding-agent/src/commands.rs`
- `crates/tau-coding-agent/src/runtime_types.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `crates/tau-coding-agent/src/self_modification_runtime.rs`
- `specs/milestones/m330/index.md`
- `specs/3637/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- `tau-extensions` remains deprecated but still supported; the goal is to keep
  that compatibility path from breaking unrelated branch validation until the
  actual migration happens.
- Any lint allowances should be as local and descriptive as possible.

## Risks / Mitigations
- Risk: broad `allow` attributes hide future real regressions.
  Mitigation: scope allowances to the minimal imports/items required and keep
  them tied to the intentional compatibility boundary.
- Risk: the first visible CI failure may hide additional downstream warnings.
  Mitigation: iterate on the local clippy command until it passes completely.

## Verification
- `cargo clippy -p tau-coding-agent --all-targets --all-features --no-deps -- -D warnings`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- If runtime files change in a way that affects targeted behavior:
  - `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c02_functional_optimizer_runs_on_update_interval -- --test-threads=1`
