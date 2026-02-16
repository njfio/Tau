# Issue 1633 Plan

Status: Reviewed

## Approach

1. Tests-first: add `scripts/dev/test-tools-rs-size-split.sh` with checks for:
   - `tools.rs` line-count threshold (<4000)
   - `tools/runtime_helpers.rs` module file presence
   - `mod runtime_helpers;` wiring in `tools.rs`
2. Run RED before extraction.
3. Extract helper block from `tools.rs` into `tools/runtime_helpers.rs` and wire imports.
4. Run GREEN split harness.
5. Run targeted parity tests:
   - `cargo test -p tau-tools tools::tests::unit_builtin_agent_tool_name_registry_includes_session_tools -- --exact`
   - `cargo test -p tau-tools tools::tests::integration_sessions_history_tool_returns_bounded_lineage -- --exact`
   - `cargo test -p tau-tools tools::tests::integration_bash_tool_dry_run_validates_without_execution -- --exact`
6. Run scoped quality checks:
   - `scripts/dev/roadmap-status-sync.sh --check --quiet`
   - `cargo fmt --check`
   - `cargo clippy -p tau-tools -- -D warnings`

## Affected Areas

- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/runtime_helpers.rs`
- `scripts/dev/test-tools-rs-size-split.sh`
- `specs/1633/spec.md`
- `specs/1633/plan.md`
- `specs/1633/tasks.md`

## Risks And Mitigations

- Risk: extraction introduces visibility/import errors.
  - Mitigation: move helpers mechanically via submodule + `use runtime_helpers::*` and run targeted parity tests.
- Risk: accidental behavior changes during refactor.
  - Mitigation: no logic edits; only file-boundary changes plus parity regression checks.

## ADR

No dependency/protocol/architecture decision changes; ADR not required.
