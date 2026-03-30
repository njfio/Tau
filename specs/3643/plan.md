# Plan: Issue #3643 - Fix tau-memory clippy derivable_impls blockers in memory runtime

## Approach
1. Use the GitHub Actions failure on PR `#3631` as RED evidence because the
   local toolchain does not currently reproduce the lint.
2. Replace the manual `Default` impl on `MemoryRelationType` with
   `#[derive(Default)]` and mark `RelatedTo` as `#[default]`.
3. Replace the manual `Default` impl on `MemoryType` with `#[derive(Default)]`
   and mark `Observation` as `#[default]`.
4. Rerun local `tau-memory` clippy for smoke coverage, then rerun the exact
   `fast-validate --base <sha>` reproduction and push the result back to
   PR `#3631`.

## Affected Areas
- `crates/tau-memory/src/runtime.rs`
- `specs/milestones/m330/index.md`
- `specs/3643/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- The derive-based defaults must preserve the existing effective defaults:
  `MemoryRelationType::RelatedTo` and `MemoryType::Observation`.

## Risks / Mitigations
- Risk: local clippy may not reproduce the CI-only lint, making regressions
  harder to confirm before push.
  Mitigation: use the GitHub log as RED evidence, keep the change purely
  mechanical, and rerun the exact PR validation path after the edit.
- Risk: the next blocker may surface only after `tau-memory` passes in CI.
  Mitigation: keep the branch under observation after the push and peel the
  next concrete failure immediately.

## Verification
- `cargo clippy -p tau-memory --all-targets --all-features --no-deps -- -D warnings`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
- PR `#3631` GitHub Actions rerun
