# Tasks: Issue #3583 - Runtime timeout, cancel, and partial-output recovery semantics

## Runtime timeout partial-output recovery slice

- [ ] T1 Specify: define runtime timeout behavior for streamed gateway turns, including pending-tool finalization, partial output preservation, and operator-visible timeout state.
- [x] T2 RED: add a gateway stream regression proving partial-output recovery emits an operator snapshot before the legacy `response.failed` compatibility frame.
- [x] T3 GREEN: emit a timeout `response.operator_turn_state.snapshot` with `status=timed_out`, `error.reason_code=gateway_timeout`, the partial assistant output observed before timeout, and any finalized pending tool context.
- [x] T4 COMPAT: keep legacy `response.failed` and pending `response.tool_execution.completed` timeout frames compatible for existing clients.
- [x] T5 CLOSEOUT: verify scoped gateway tests, format/lint, Cargo manifest stability, and publish #3583 evidence.

## Runtime recovery learning bulletin replay slice

- [x] T6 SPECIFY: timeout and cancellation recovery outcomes must become reusable
	action-history learning signals whose reason-code evidence appears in later Ralph
	`## Learning Insights` bulletins.
- [x] T7 RED: add gateway coverage proving a follow-up request after a timed-out or
	cancelled streamed mission expects a runtime recovery learning bulletin with
	`gateway_runtime` and exact `gateway_timeout` or `gateway_cancelled` reason-code
	evidence.
- [x] T8 GREEN: persist timeout and cancellation outcomes into the gateway learning path
	without reusing `complete_task` or `gateway_verifier`, and render the reason-code
	evidence through existing failure-pattern bulletin formatting.
- [x] T9 CLOSEOUT: verify timeout/cancellation operator snapshots, verifier-blocked
	learning replay compatibility, formatting, clippy, Cargo manifest stability, and
	publish #3583 evidence.

Learning boundary:

- `gateway_runtime` is the action-history learning signal name for runtime-owned
	timeout and cancellation failures that are not normal observed tools, not explicit
	mission completion signals, and not verifier-blocked fail-closed decisions.
- `reason_code` is part of the replay contract: later `Learning Insights` bulletins
	must preserve `gateway_timeout` and `gateway_cancelled` evidence so the next Ralph
	turn can avoid repeating timeout-prone or cancellation-prone recovery behavior.
- `complete_task` remains reserved for explicit mission completion signals.
- `gateway_verifier` remains reserved for verifier-blocked fail-closed recovery evidence.

## Semantics

- Runtime timeout with no tool progress remains a closed failure and marks the mission blocked.
- Runtime timeout after partial output should preserve the partial output in the operator snapshot instead of making operators reconstruct it from legacy deltas.
- Pending tools finalized during timeout should remain visible as failed tool context with `timed_out=true` in legacy frames and failed tool state in operator snapshots.
- The operator snapshot is additive: clients that only understand `response.failed` keep working.
- Runtime cancellation should mirror timeout recovery where practical: preserve buffered partial assistant output, finalize pending tool state as cancelled/failed context, emit an operator-visible `cancelled` snapshot before compatibility failure frames, and let clients suppress duplicate generic gateway errors.

## Evidence

- `cargo test -p tau-gateway runtime_timeout_partial_output_recovery -- --test-threads=1`
	proves streamed timeouts emit a timed-out operator snapshot with buffered partial assistant output before `response.failed`.
- `cargo test -p tau-gateway mission_completion_outcome_snapshot -- --test-threads=1`
	keeps completion-outcome snapshot semantics stable.
- `cargo test -p tau-gateway operator_turn_state_recovery_policy_snapshot -- --test-threads=1`
	keeps verifier-blocked recovery snapshots stable.
- `cargo test -p tau-tui operator_turn_state_checkpoint_blocked_timeout -- --test-threads=1`
	proves the TUI preserves timeout partial assistant output from a timed-out operator snapshot and suppresses the following generic `response.failed` duplicate.
- `cargo test -p tau-gateway runtime_cancel -- --test-threads=1`
	proves a cancelled streamed turn emits an operator snapshot with buffered partial output before legacy compatibility failure frames.
- `cargo test -p tau-gateway runtime_recovery_learning_bulletin -- --test-threads=1`
	proves a cancelled runtime recovery outcome persists as `gateway_runtime` learning evidence and replays `gateway_cancelled` into a follow-up `## Learning Insights` bulletin.
- `cargo test -p tau-gateway verifier_blocked_learning_bulletin -- --test-threads=1`
	keeps the separate `gateway_verifier` fail-closed learning replay contract intact.
- `cargo fmt --check`
	keeps the workspace format-clean after timeout/cancellation recovery changes.
- `cargo clippy -p tau-gateway -p tau-tui --tests --no-deps -- -D warnings`
	keeps gateway and TUI compatibility paths warning-free.
- `git diff --quiet -- Cargo.toml`
	confirms the cancellation recovery slice did not mutate workspace manifests.
