# Plan: Issue #3678 - Add prompt history and editor ergonomics to `tau-tui` REPL

Status: Implemented

## Approach
Implement a local editor-history model in the TUI and extend the multiline
editor with shell-style movement/deletion helpers. Keep the change inside
`crates/tau-tui/src/interactive` and avoid any gateway contract changes.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - store submitted prompt history, current history cursor, and unsent draft
  - push sent prompts into history on successful submit path
- `crates/tau-tui/src/interactive/input.rs`
  - add text replacement helpers and shell-style movement/deletion primitives
- `crates/tau-tui/src/interactive/app_keys.rs`
  - bind history recall and editor controls in insert mode
- `crates/tau-tui/src/interactive/ui_status.rs`
  - surface the new editor shortcuts in the help line
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - update the help overlay to document the new editor/history controls
- `crates/tau-tui/src/interactive/*tests*.rs`
  - add RED tests for history recall, draft preservation, and editor controls

## Contracts
- Prompt history is session-local in the TUI process; it does not persist to
  gateway state
- Recalled history entries populate the existing input editor and remain fully
  editable before send
- The newest history slot represents the editable draft buffer
- Multiline send/newline behavior remains unchanged

## Risks
- Insert-mode key bindings must avoid clashing with existing global shortcuts
- History navigation must handle the empty-history and single-entry cases cleanly
- Editor helpers should remain Unicode-safe by operating on character indices,
  not byte offsets

## Verification Strategy
- Add failing tests first for history recall, draft restore, and editor
  movement/deletion helpers
- Re-run existing TUI gateway tests to protect send/newline behavior
- Build `tau-tui` after the scoped tests pass
