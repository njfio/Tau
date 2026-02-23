# Plan: Issue #3412

## Approach
1. Reproduce compile failures with `cargo test -p tau-memory` in clean worktree (RED evidence), then follow dependent compilation until all unsigned SQLite binding sites are identified.
2. Locate all `rusqlite` bind/read call sites that pass `u64` directly.
3. Replace those call sites with explicit checked conversion to SQLite-compatible integer type (`i64`) and checked back-conversion where needed.
4. Add/adjust regression tests for conversion boundaries and fail-closed behavior.
5. Run targeted crate tests (`tau-memory`, `tau-session`) and a dependent compile path as regression proof.
6. Update tasks artifact with RED/GREEN/REGRESSION evidence and tier matrix.

## Affected Modules
- `crates/tau-memory/src/runtime.rs`
- `crates/tau-memory/src/runtime/backend.rs`
- `crates/tau-memory/src/runtime/query.rs`
- `crates/tau-session/src/session_storage.rs`
- `specs/3412/spec.md`
- `specs/3412/plan.md`
- `specs/3412/tasks.md`
- `specs/milestones/m292/index.md`

## Risks and Mitigations
- Risk: accidental timestamp truncation or sign issues.
  - Mitigation: use existing checked conversion helpers and add regression assertions around conversion errors.
- Risk: hidden call sites still binding unsigned types directly.
  - Mitigation: search for relevant `params!` and `row.get` sites across touched runtime modules and re-run compile/tests.

## Interfaces / Contracts
- No API/wire-format changes.
- Internal SQLite integer conversion contract tightened to explicit checked conversion.

## ADR
Not required (no dependency, architecture, or protocol change).
