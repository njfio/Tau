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
