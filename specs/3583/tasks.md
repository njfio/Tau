# Tasks: Issue #3583 - Runtime timeout, cancel, and partial-output recovery semantics

## Runtime timeout partial-output recovery slice

- [ ] T1 Specify: define runtime timeout behavior for streamed gateway turns, including pending-tool finalization, partial output preservation, and operator-visible timeout state.
- [x] T2 RED: add a gateway stream regression proving partial-output recovery emits an operator snapshot before the legacy `response.failed` compatibility frame.
- [x] T3 GREEN: emit a timeout `response.operator_turn_state.snapshot` with `status=timed_out`, `error.reason_code=gateway_timeout`, the partial assistant output observed before timeout, and any finalized pending tool context.
- [x] T4 COMPAT: keep legacy `response.failed` and pending `response.tool_execution.completed` timeout frames compatible for existing clients.
- [ ] T5 CLOSEOUT: verify scoped gateway tests, format/lint, Cargo manifest stability, and publish #3583 evidence.

## Semantics

- Runtime timeout with no tool progress remains a closed failure and marks the mission blocked.
- Runtime timeout after partial output should preserve the partial output in the operator snapshot instead of making operators reconstruct it from legacy deltas.
- Pending tools finalized during timeout should remain visible as failed tool context with `timed_out=true` in legacy frames and failed tool state in operator snapshots.
- The operator snapshot is additive: clients that only understand `response.failed` keep working.

## Evidence

- `cargo test -p tau-gateway runtime_timeout_partial_output_recovery -- --test-threads=1`
	proves streamed timeouts emit a timed-out operator snapshot with buffered partial assistant output before `response.failed`.
- `cargo test -p tau-gateway mission_completion_outcome_snapshot -- --test-threads=1`
	keeps completion-outcome snapshot semantics stable.
- `cargo test -p tau-gateway operator_turn_state_recovery_policy_snapshot -- --test-threads=1`
	keeps verifier-blocked recovery snapshots stable.
- `cargo test -p tau-tui operator_turn_state_checkpoint_blocked_timeout -- --test-threads=1`
	proves the TUI preserves timeout partial assistant output from a timed-out operator snapshot and suppresses the following generic `response.failed` duplicate.
