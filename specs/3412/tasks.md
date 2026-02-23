# Tasks: Issue #3412 - tau-memory rusqlite compatibility fix

- [x] T1 (RED): reproduce compile failure in clean worktree and capture failing command/output.
- [x] T2 (GREEN): patch `tau-memory` and `tau-session` SQLite bind/read call sites to use checked SQLite-compatible integer conversions.
- [x] T3 (REGRESSION): add/adjust regression coverage for conversion boundaries and fail-closed behavior.
- [x] T4 (VERIFY): run targeted tests and dependent compile path; record RED/GREEN/REGRESSION evidence and tier outcomes.

## Verification Evidence (2026-02-23)

### RED

- `cargo test -p tau-memory`
  - failed with unsigned SQLite trait-bound errors:
    - `crates/tau-memory/src/runtime/backend.rs:218` (`u64: ToSql`)
    - `crates/tau-memory/src/runtime.rs:791` (`u64: ToSql`)
    - `crates/tau-memory/src/runtime.rs:990` (`u64: FromSql`)
    - `crates/tau-memory/src/runtime.rs:1010` (`u64: ToSql`)
- `cargo test -p tau-tools --lib` (dependent compile chain)
  - exposed additional unsigned SQLite trait-bound errors in `tau-session`:
    - `crates/tau-session/src/session_storage.rs:127` (`u64: FromSql`)
    - `crates/tau-session/src/session_storage.rs:413` (`u64: FromSql`)
    - `crates/tau-session/src/session_storage.rs:414` (`Option<u64>: FromSql`)
    - `crates/tau-session/src/session_storage.rs:445` (`u64: ToSql`)

### GREEN

- Patched modules:
  - `crates/tau-memory/src/runtime.rs`
  - `crates/tau-memory/src/runtime/backend.rs`
  - `crates/tau-memory/src/runtime/query.rs`
  - `crates/tau-session/src/session_storage.rs`
- Added regression tests:
  - `runtime::backend::tests::regression_spec_3412_checkpoint_upsert_rejects_updated_unix_ms_outside_sqlite_integer_range`
  - `runtime::backend::tests::regression_spec_3412_checkpoint_upsert_rejects_chunk_index_outside_sqlite_integer_range`
  - `runtime::tests::regression_spec_3412_sqlite_i64_from_u64_rejects_values_outside_sqlite_integer_range`
  - `runtime::tests::regression_spec_3412_sqlite_u64_from_i64_rejects_negative_sqlite_integer_values`

### REGRESSION

- `cargo test -p tau-memory` -> `90 passed; 0 failed`
- `cargo test -p tau-session` -> `80 passed; 0 failed`
- `cargo test -p tau-tools tool_policy_config::tests::unit_tool_policy_json_exposes_protected_path_controls -- --exact --test-threads=1` -> `1 passed; 0 failed`
  - used as dependent compile-path proof after clearing memory/session unsigned SQLite compile errors.
- Hygiene:
  - `cargo fmt --check` -> passed
  - `cargo clippy -p tau-memory -p tau-session -- -D warnings` -> passed

### Test Tier Matrix

| Tier | ✅/❌/N/A | Tests | N/A Why |
|---|---|---|---|
| Unit | ✅ | `cargo test -p tau-memory`, `cargo test -p tau-session` | |
| Property | N/A | no property/invariant expansion needed for this integer binding fix | no algorithmic fuzzable invariant change |
| Contract/DbC | N/A | no contract-attribute interface changes | no DbC surface touched |
| Snapshot | N/A | no snapshot artifacts changed | no snapshot outputs in scope |
| Functional | ✅ | existing persistence/read flows in `tau-memory` + `tau-session` test suites | |
| Conformance | ✅ | C-01..C-04 mapped in `specs/3412/spec.md` and executed by verify commands | |
| Integration | ✅ | dependent compile path via scoped `tau-tools` selector run | |
| Fuzz | N/A | no new untrusted parser surface introduced | fuzz tracked separately |
| Mutation | N/A | not executed for this delta | follow-up can run when broader workspace baseline is stabilized |
| Regression | ✅ | new `regression_spec_3412_*` tests in memory backend/runtime | |
| Performance | N/A | no performance-path behavior changes | no benchmark scope in this issue |
