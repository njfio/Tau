# Plan: Issue #3557 - Interactive in-flight progress indicator for TUI agent turns

Status: Implemented

## Approach
1. Add a lightweight runtime-loop progress tracker in
   `crates/tau-coding-agent/src/runtime_loop.rs`:
   - start line before prompt dispatch,
   - periodic heartbeat loop while in-flight,
   - end line on terminal status.
2. Gate progress tracker to TTY interactive mode only
   (`stdin.is_terminal() && stdout.is_terminal()`).
3. Integrate tracker in both interactive prompt flows:
   - standard prompt routing path,
   - plan-first orchestrator path.
4. Add tests:
   - runtime-loop unit tests for TTY-gating helper and status formatting.
   - REPL harness regression asserting non-tty runs do not emit
     `interactive.turn=` lines.
5. Update docs with indicator behavior so operators know what to expect.

## Affected Modules
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/tests/cli_integration/repl_harness.rs`
- `crates/tau-coding-agent/testdata/repl-harness/*.json` (if fixture changes needed)
- `README.md` and/or `docs/guides/operator-deployment-guide.md`

## Risks / Mitigations
- Risk: heartbeat output interleaves poorly with streamed tokens.
  - Mitigation: heartbeat to stderr with concise line format and bounded cadence.
- Risk: noise in scripted pipelines.
  - Mitigation: strict TTY gating.
- Risk: async task leakage for heartbeat loop.
  - Mitigation: explicit stop signal + bounded join timeout.

## Interfaces / Contracts
- New stderr progress lines (TTY interactive only):
  - `interactive.turn=start timeout_ms=<...>`
  - `interactive.turn=running elapsed_ms=<...>`
  - `interactive.turn=end status=<...> elapsed_ms=<...>`

## ADR
No ADR required (runtime UX observability enhancement; no dependency/protocol
change).

## Execution Summary
1. Added TTY-gated interactive progress helpers in
   `crates/tau-coding-agent/src/runtime_loop.rs`:
   - `should_emit_interactive_turn_progress`
   - `format_interactive_turn_start_line`
   - `format_interactive_turn_running_line`
   - `format_interactive_turn_end_line`
2. Added `InteractiveTurnProgressTracker` with:
   - start marker on turn begin,
   - heartbeat every 2s with elapsed ms,
   - terminal status marker on end.
3. Wired tracker into interactive prompt execution paths:
   - standard prompt route
   - plan-first route
4. Added/updated tests:
   - runtime-loop unit tests for gating + line contract
   - REPL harness fixture guards to keep non-tty output clean
5. Updated operator docs with progress marker behavior.

## Verification Notes
- `cargo test -p tau-coding-agent interactive_turn_progress -- --nocapture`
- `cargo test -p tau-coding-agent integration_repl_harness_executes_prompt_flow_with_mock_openai -- --nocapture`
- `TAU_REPL_HARNESS_TIMEOUT_MS=7000 cargo test -p tau-coding-agent regression_repl_harness_turn_timeout_remains_deterministic_in_ci -- --nocapture`
- `cargo fmt --check`
