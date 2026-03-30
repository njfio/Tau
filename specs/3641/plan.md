# Plan: Issue #3641 - Fix tau-onboarding clippy derivable_impls blocker

## Approach
1. Reproduce the current failure with:
   - `cargo clippy -p tau-onboarding --all-targets --all-features --no-deps -- -D warnings`
   - `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
2. Replace the manual `Default` impl on `SelfImprovementConfig` with
   `#[derive(Default)]`.
3. Verify that local `tau-onboarding` clippy passes and that the branch-level
   fast-validate path advances beyond `tau-onboarding`.

## Affected Areas
- `crates/tau-onboarding/src/config_file.rs`
- `specs/milestones/m330/index.md`
- `specs/3641/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- The struct’s derived default matches the current manual implementation because
  every field is `bool` and currently defaults to `false`.

## Risks / Mitigations
- Risk: a field-specific non-default initializer could be lost during the
  conversion.
  Mitigation: verify the struct fields are all plain `bool` values and compare
  the old manual impl against the derived semantics before removing it.
- Risk: the next branch blocker may surface only after `tau-onboarding` passes.
  Mitigation: rerun the exact fast-validate reproduction immediately after the
  local crate check passes.

## Verification
- `cargo clippy -p tau-onboarding --all-targets --all-features --no-deps -- -D warnings`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
