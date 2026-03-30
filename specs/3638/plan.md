# Plan: Issue #3638 - Contain tau-custom-command deprecation lint in package-scoped clippy

## Approach
1. Reproduce the current failure with:
   - `cargo clippy -p tau-custom-command --all-targets --all-features --no-deps -- -D warnings`
   - `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
2. Contain the lint at the deprecated crate boundary rather than on each
   individual type or usage site. The crate already declares itself as a
   deprecated compatibility surface in `lib.rs`; the same boundary is the
   narrowest honest place to suppress self-referential deprecation warnings.
3. Verify that local crate clippy passes and that the branch-level
   fast-validate path advances beyond `tau-custom-command`.

## Affected Areas
- `crates/tau-custom-command/src/lib.rs`
- `specs/milestones/m330/index.md`
- `specs/3638/`

## Compatibility / Contract Notes
- No behavioral or API changes are intended.
- The crate remains deprecated; this change only prevents that intentional
  deprecation marker from breaking unrelated branch validation.
- The allowance should be limited to `deprecated`, not a broader warnings
  suppression.

## Risks / Mitigations
- Risk: a crate-level allowance could hide newly introduced deprecated usage
  that is not part of the intended compatibility surface.
  Mitigation: scope the allowance to `deprecated` only, keep it on the already
  deprecated crate root, and avoid suppressing other lint categories.
- Risk: the next branch blocker may surface only after this crate passes.
  Mitigation: rerun the exact fast-validate reproduction immediately after the
  local crate check passes.

## Verification
- `cargo clippy -p tau-custom-command --all-targets --all-features --no-deps -- -D warnings`
- `cargo test -p tau-custom-command --lib custom_command_contract::tests::spec_c01_custom_command_contract_capabilities_cover_expected_sets`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
