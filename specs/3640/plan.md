# Plan: Issue #3640 - Contain tau-tools extension compatibility deprecation lint

## Approach
1. Reproduce the current failure with:
   - `cargo clippy -p tau-tools --all-targets --all-features --no-deps -- -D warnings`
   - `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
2. Identify the exact bridge points where `tau-tools` still consumes
   `tau-extensions` compatibility types and functions.
3. Contain deprecation warnings locally at those bridge points:
   - deprecated imports in `tools.rs` and `bash_tool.rs`
   - the extension-tool registration adapter in `registry_core.rs`
   - the compatibility regression test in `tests.rs`
4. Verify that local `tau-tools` clippy passes and that the branch-level
   fast-validate path advances beyond `tau-tools`.

## Affected Areas
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/bash_tool.rs`
- `crates/tau-tools/src/tools/tests.rs`
- `specs/milestones/m330/index.md`
- `specs/3640/`

## Compatibility / Contract Notes
- No behavior changes are intended.
- `tau-tools` still needs to bridge into `tau-extensions` until the actual
  migration lands; this task only prevents that temporary bridge from breaking
  package-scoped validation.
- Containment should stay item-local rather than crate-wide.

## Risks / Mitigations
- Risk: overly broad allowances could hide future deprecated usage unrelated to
  the known extension bridge.
  Mitigation: scope `allow(deprecated)` to the bridge imports, structs,
  functions, and tests that directly reference `tau-extensions`.
- Risk: the next branch blocker may surface only after `tau-tools` passes.
  Mitigation: rerun the exact fast-validate reproduction immediately after the
  local crate check passes.

## Verification
- `cargo clippy -p tau-tools --all-targets --all-features --no-deps -- -D warnings`
- `cargo test -p tau-tools --lib tools::tests::integration_tool_builder_generated_tool_executes_through_extension_runtime -- --test-threads=1`
- `./scripts/dev/fast-validate.sh --base 36dd1b5e417c68d6e8e49c276e3fccc297c502eb`
