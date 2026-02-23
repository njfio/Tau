# Spec: Issue #3412 - Fix rusqlite u64 sqlite binding regressions in memory/session runtime paths

Status: Reviewed

## Problem Statement
In clean builds from `origin/master`, runtime persistence modules fail to compile because some SQLite bind/read call sites pass `u64`/`usize` directly to `rusqlite` `params!`/`row.get`, while the active `rusqlite` trait surface does not implement `ToSql`/`FromSql` for those unsigned types. Reproduced failures affected both:
- `tau-memory` (`crates/tau-memory/src/runtime*.rs`)
- `tau-session` (`crates/tau-session/src/session_storage.rs`)

## Scope
In scope:
- Normalize SQLite integer bind/read paths in `tau-memory` and `tau-session` to use supported integer types with explicit checked conversion.
- Preserve existing runtime semantics for memory record timestamps.
- Preserve existing runtime semantics for session lineage persistence.
- Add regression coverage proving compatibility and fail-closed conversion behavior.
- Capture RED/GREEN/REGRESSION evidence in issue tasks artifact.

Out of scope:
- Schema redesign or migration of persisted timestamp units.
- Behavioral changes to memory ranking/retrieval logic beyond compile-compatibility adjustments.

## Acceptance Criteria
### AC-1 Compile compatibility restored for memory/session SQLite persistence/read paths
Given a clean worktree from `origin/master`,
when `tau-memory` and `tau-session` are compiled,
then SQLite persistence/read code no longer emits unsigned integer `ToSql`/`FromSql` trait bound errors.

### AC-2 Runtime behavior remains stable for timestamp/session persistence paths
Given memory record and session lineage write/read flows,
when records are persisted and reloaded,
then timestamp and session id/parent semantics remain unchanged and deterministic.

### AC-3 Conversion failure handling is explicit and fail-closed
Given out-of-range integer conversion inputs for SQLite integer fields,
when conversion helpers execute,
then deterministic conversion errors are surfaced without silent truncation.

### AC-4 Verification evidence is documented
Given AGENTS testing contract,
when verification runs complete,
then RED/GREEN/REGRESSION evidence and tier outcomes are recorded in `specs/3412/tasks.md`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Unit/Compile | clean `tau-memory` crate source | run `cargo test -p tau-memory` | compile succeeds without unsigned SQLite trait-bound errors |
| C-02 | AC-1/AC-2 | Unit/Compile | clean `tau-session` crate source | run `cargo test -p tau-session` | compile + session runtime tests pass with unchanged lineage semantics |
| C-03 | AC-3 | Regression | oversized integer conversion inputs | call SQLite conversion helpers | deterministic fail-closed error is returned |
| C-04 | AC-4 | Process | verification commands and outputs | update tasks artifact | RED/GREEN/REGRESSION evidence is present |

## Success Metrics / Observable Signals
- `cargo test -p tau-memory` passes in clean worktree.
- `cargo test -p tau-session` passes in clean worktree.
- A transitive compile path that includes memory/session/runtime dependencies executes without unsigned SQLite compile errors (verified via scoped `tau-tools` selector run).
- `specs/3412/spec.md`, `specs/3412/plan.md`, and `specs/3412/tasks.md` are present and updated through lifecycle.
