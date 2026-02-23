# M292 - Tau Memory rusqlite u64 compatibility hardening

Status: InProgress

## Context
Clean builds from `origin/master` currently fail in runtime persistence/query modules due `rusqlite` trait bounds for unsigned integer bindings (`u64`/`usize`) on SQLite paths. Reproduced failures affect `tau-memory` and `tau-session`, and block deterministic verification runs that compile these crates transitively while executing conformance selectors.

## Scope
- Execute issue `#3412` as the implementation task for this milestone.
- Restore clean compile/test behavior for memory/session SQLite bindings without changing external runtime contracts.
- Add/adjust regression coverage for integer conversion behavior and document verification evidence.

## Linked Issues
- Task: #3412

## Success Signals
- `cargo test -p tau-memory` passes on clean worktree.
- `cargo test -p tau-session` passes on clean worktree.
- Dependent compile paths that include memory/session crates no longer fail with unsigned SQLite trait-bound errors.
- Issue artifacts exist under `specs/3412/` and are advanced through lifecycle states.
