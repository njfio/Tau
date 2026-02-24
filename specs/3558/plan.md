# Plan: Issue #3558 - Interactive start marker timeout clarity

Status: Implemented

## Approach
1. Update runtime-loop start marker formatter contract to emit:
   `interactive.turn=start turn_timeout_ms=<...> request_timeout_ms=<...>`.
2. Ensure all `InteractiveRuntimeConfig` constructors provide
   `request_timeout_ms`.
3. Update existing unit tests that validate marker line formatting.
4. Update operator docs (`README` and deployment guide) to describe the new
   line format.

## Affected Modules
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `README.md`
- `docs/guides/operator-deployment-guide.md`

## Risks / Mitigations
- Risk: Partial propagation of new field in test helpers causing build breaks.
  - Mitigation: compile and run targeted `tau-coding-agent` tests touching
    interactive runtime.
- Risk: Docs drift from actual output.
  - Mitigation: update docs in same change set as tests.

## Interfaces / Contracts
- Interactive start marker contract becomes:
  `interactive.turn=start turn_timeout_ms=<u64> request_timeout_ms=<u64>`

## ADR
No ADR required (small runtime observability contract correction).

## Execution Summary
1. Updated start-line formatter contract in
   `crates/tau-coding-agent/src/runtime_loop.rs` to emit both timeout fields.
2. Wired `request_timeout_ms` into `InteractiveRuntimeConfig` construction in
   `crates/tau-coding-agent/src/startup_local_runtime.rs`.
3. Updated runtime-loop unit contract test to assert dual-timeout start marker.
4. Updated docs in `README.md` and
   `docs/guides/operator-deployment-guide.md`.
5. Verified with focused `tau-coding-agent` tests and `cargo fmt --check`.
