# Tasks: Issue #3581 - Define shared operator turn/task state protocol for TUI and webchat

- [x] T1 (RED): create #3581 spec artifacts, ADR, and RED tau-contract operator-state contract tests.
  - Verify RED with: `bash -c '! cargo test -p tau-contract operator_state -- --test-threads=1'`

- [x] T2 (ADR): accept the shared operator turn/task state v1 architecture decision.
  - ADR path: `docs/adrs/0005-shared-operator-turn-task-state.md`

- [x] T3 (GREEN): implement additive `OperatorTurnState` v1 types and fixtures in `tau-contract`.
  - Verify with: `cargo test -p tau-contract operator_state -- --test-threads=1`

- [x] T4 (DOCS): document runtime mappings, TUI consumption, webchat consumption, and backwards compatibility.
  - Doc path: `docs/architecture/shared-operator-state-v1.md`

- [x] T5 (VERIFY/CLOSE): run final scoped gates, path-limit commit/push, close issue #3581, and record the reusable pattern.
  - `cargo test -p tau-contract operator_state -- --test-threads=1`
  - `cargo fmt --check`
  - `cargo clippy -p tau-contract --tests --no-deps -- -D warnings`
  - `git diff --quiet -- Cargo.toml Cargo.lock`
