# Tasks: Issue #3582 - Redesign Tau TUI as a transcript-first operator terminal

- [x] T1 (RED): create #3582 spec artifacts, label the issue, and map TUI/gateway/shared-contract touchpoints.

- [x] T2 (RED): add failing operator-state adapter tests for transcript-first TUI consumption.
  - Verify RED with: `bash -c '! cargo test -p tau-tui operator_state -- --test-threads=1'`

- [x] T3 (GREEN): implement additive `OperatorTurnState` consumption in tau-tui.
  - Add internal `tau-contract` dependency to `crates/tau-tui/Cargo.toml` if needed.
  - Preserve existing gateway SSE behavior.
  - Verify with: `cargo test -p tau-tui operator_state -- --test-threads=1`

- [x] T4 (DOCS/CLOSEOUT): document the adapter boundary, run final gates, path-limit commit/push, and update issue #3582 with evidence.
  - `cargo test -p tau-tui operator_state -- --test-threads=1`
  - `cargo fmt --check`
  - `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`
  - `git diff --quiet -- Cargo.toml`

- [x] T5 (RED): record the additive `response.operator_turn_state.snapshot` SSE decision and add failing live-stream consumption tests.
  - Verify RED with: `bash -c '! cargo test -p tau-tui operator_turn_state_snapshot -- --test-threads=1'`

- [x] T6 (GREEN): parse `response.operator_turn_state.snapshot` frames in the TUI gateway client and apply them through `OperatorTurnState` consumption.
  - Preserve existing `response.*` delta, tool, completion, and failure frame behavior.
  - Verify with: `cargo test -p tau-tui operator_turn_state_snapshot -- --test-threads=1`

- [x] T7 (DOCS): update the TUI operator-state architecture note with the live snapshot event boundary and compatibility rules.

- [x] T8 (CLOSEOUT): run final tau-tui gates, path-limit commit/push, and comment issue #3582 with evidence for the live snapshot slice.

- [x] T9 (RED): map gateway emission touchpoints and add failing tau-gateway tests for additive `response.operator_turn_state.snapshot` emission.
  - Verify RED with: `bash -c '! cargo test -p tau-gateway operator_turn_state_snapshot -- --test-threads=1'`

- [x] T10 (GREEN): emit `response.operator_turn_state.snapshot` from gateway streaming responses without removing legacy `response.*` frames.
  - Verify with: `cargo test -p tau-gateway operator_turn_state_snapshot -- --test-threads=1`

- [x] T11 (COMPAT): verify tau-gateway and tau-tui snapshot paths together and preserve root manifest stability.

- [x] T12 (CLOSEOUT): path-limit commit/push the gateway emission slice and comment issue #3582 with evidence.

- [x] T13 (RED): map transcript-first layout boundaries and add failing tau-tui render tests for the operator shell layout.
  - Verify RED with: `bash -c '! cargo test -p tau-tui transcript_first_layout -- --test-threads=1'`

- [x] T14 (GREEN): refine the TUI render layer so transcript content stays primary while status, input, help, and tool progress remain stable.
  - Verify with: `cargo test -p tau-tui transcript_first_layout -- --test-threads=1`

- [x] T15 (COMPAT): keep existing operator-state snapshot and gateway streaming render behavior green alongside the new layout tests.

- [x] T16 (CLOSEOUT): path-limit commit/push the layout slice and comment issue #3582 with evidence.

- [x] T17 (RED): map richer live snapshot tool/failure and turn-keyed reconciliation boundaries, then add failing gateway/TUI tests.
  - Verify RED with: `bash -c '! cargo test -p tau-gateway operator_turn_state_tool_failure_snapshot -- --test-threads=1'`
  - Verify RED with: `bash -c '! cargo test -p tau-tui operator_turn_state_snapshot_turn_keyed -- --test-threads=1'`

- [x] T18 (GREEN): emit richer live OperatorTurnState snapshots and reconcile TUI snapshots by turn identity without duplicating legacy deltas or tool rows.
  - Verify with: `cargo test -p tau-gateway operator_turn_state_tool_failure_snapshot -- --test-threads=1`
  - Verify with: `cargo test -p tau-tui operator_turn_state_snapshot_turn_keyed -- --test-threads=1`

- [x] T19 (COMPAT): keep existing operator-state, snapshot, transcript-first layout, and gateway emission tests green after richer snapshot reconciliation.

- [x] T20 (CLOSEOUT): path-limit commit/push the richer snapshot slice and comment issue #3582 with verification evidence.
