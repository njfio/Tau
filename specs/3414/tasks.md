# Tasks: Issue #3414 - README examples marker regression

1. [x] T1 (RED, Regression): reproduce failing README marker test with `cargo test -p tau-coding-agent --test examples_assets`.
2. [x] T2 (GREEN, Docs): update `README.md` to include all expected `./examples/...` path markers.
3. [x] T3 (RED, Regression): capture failing `tau-tools` memory test evidence from full-workspace run.
4. [x] T4 (GREEN, Test determinism): update `test_policy_with_memory` in `crates/tau-tools/src/tools/tests.rs` for deterministic local-hash embedding config.
5. [x] T5 (Verify): rerun failing targets (`examples_assets`, affected `tau-tools` tests) and `cargo test` to confirm blockers are removed.
6. [x] T6 (Governance): set spec status to `Implemented` and post issue process-log/closure evidence.

## RED / GREEN Evidence

### RED
- `cargo test -p tau-coding-agent --test examples_assets`
  - failed with: `README should reference ./examples/starter/package.json`
- `cargo test` (pre-fix run)
  - failed in `tau-tools` with:
    - `spec_2444_c05_legacy_records_without_relations_return_stable_defaults`
    - `integration_memory_search_tool_honors_scope_filter`
    - `integration_memory_tools_fixture_roundtrip_is_deterministic`

### GREEN
- `cargo test -p tau-coding-agent --test examples_assets`
- `cargo test -p tau-tools --lib spec_2444_c05_legacy_records_without_relations_return_stable_defaults`
- `cargo test -p tau-tools --lib integration_memory_search_tool_honors_scope_filter`
- `cargo test -p tau-tools --lib integration_memory_tools_fixture_roundtrip_is_deterministic`
- `cargo test -p tau-tools --lib`
- `cargo test`

### Regression
- `cargo fmt --all -- --check`
- `cargo clippy -- -D warnings`
