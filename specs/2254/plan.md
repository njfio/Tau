# Plan #2254

Status: Reviewed
Spec: specs/2254/spec.md

## Approach

1. Extend `tau-session` storage/runtime with a persisted usage ledger:
   - add a session usage summary model
   - read/write summary atomically next to the session store
   - expose store methods to read and record deltas
2. Extend `SessionStats` in `tau-session` to include usage/cost totals and
   render them in text/json outputs.
3. Wire `tau-coding-agent` prompt loop to record usage/cost deltas on successful
   prompt completion by diffing pre/post `Agent::cost_snapshot()`.
4. Add tests covering ledger persistence, stats rendering, and runtime wiring.

## Affected Modules

- `crates/tau-session/src/lib.rs`
- `crates/tau-session/src/session_storage.rs`
- `crates/tau-session/src/session_store_runtime.rs`
- `crates/tau-session/src/session_commands.rs`
- `crates/tau-session/src/tests.rs`
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/session_and_doctor.rs`

## Risks and Mitigations

- Risk: session format regression for existing JSONL/SQLite sessions.
  - Mitigation: keep message storage format unchanged and persist usage in a
    separate ledger artifact.
- Risk: double counting usage on cancelled/timed out turns.
  - Mitigation: record deltas only after successful prompt completion.
- Risk: floating-point formatting instability in assertions.
  - Mitigation: compare numeric json values and use tolerant float checks.

## Interfaces / Contracts

- `SessionStore` records usage deltas with saturating token arithmetic.
- `SessionStats` remains backward compatible while adding new fields for usage.
- `/session-stats` command remains `usage: /session-stats [--json]`.
