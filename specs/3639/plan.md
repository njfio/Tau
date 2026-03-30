# Plan: Issue #3639 - Contain tau-extensions deprecation lint in package-scoped clippy

## Approach
1. Reproduce the current failure with:
   - `cargo clippy -p tau-extensions --all-targets --all-features --no-deps -- -D warnings`
   - `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
2. Contain the lint at the deprecated crate boundary rather than on each
   individual type or usage site. The crate already declares itself as a
   deprecated compatibility surface in `lib.rs`; the same boundary is the
   narrowest honest place to suppress self-referential deprecation warnings.
3. Verify that local crate clippy passes and that the branch-level
   fast-validate path advances beyond `tau-extensions`.

## Affected Areas
- `crates/tau-extensions/src/lib.rs`
- `specs/milestones/m330/index.md`
- `specs/3639/`

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
- `cargo clippy -p tau-extensions --all-targets --all-features --no-deps -- -D warnings`
- `cargo test -p tau-extensions --lib tests::integration_list_extension_manifests_reports_valid_and_invalid_entries -- --test-threads=1`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
