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
