# Plan: Issue #3680 - Add local session persistence to `tau-tui` REPL

Status: Implemented

## Approach
Add a small serializable session-state snapshot for the interactive TUI and
load/save it around the existing `run_interactive` lifecycle. Keep persistence
local to `tau-tui` and avoid any gateway contract changes.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - expose snapshot/load helpers for draft input, prompt history, and mission
    binding
- `crates/tau-tui/src/interactive/app_runtime.rs`
  - load local state before entering the event loop and persist it during
    interaction / shutdown
- `crates/tau-tui/src/interactive/mod.rs`
  - add a persistence module if needed
- `crates/tau-tui/src/main.rs`
  - provide a default interactive local-state path through `AppConfig`
- new `crates/tau-tui/src/interactive/session_state.rs`
  - file-backed snapshot type and read/write helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs` or a new focused test
  module
  - RED/green persistence tests

## Contracts
- Default local state path: `.tau/tui/interactive-session.json`
- Persisted fields:
  - input draft
  - prompt history
  - gateway session key
  - active mission id
- Missing or invalid state returns a clean default app state
- Persistence should not be required for the TUI to run

## Risks
- Saving on every interaction can be noisy; use a narrow persistence point that
  still keeps state durable
- The snapshot format should be forward-tolerant and optional-field friendly
- Restoring mission/session binding must not accidentally trigger gateway calls

## Verification Strategy
- Add failing tests first for restore, save, and invalid-state fallback
- Re-run existing `interactive::app_gateway_tests`
- Build `tau-tui` after the scoped tests pass
