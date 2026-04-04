# Plan: Issue #3699 - Add Left/Right cursor movement and Delete key to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette editor with character-wise cursor movement helpers
and a forward-delete helper, then route `Left`, `Right`, and `Delete` through
the existing command-palette key handler. Keep the slice limited to basic
cursor editing so it cleanly layers on top of the line-wise and word-wise
editing already shipped in M335.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add command-palette helpers for moving left/right by one character and
    deleting the character at the current cursor
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `Left`, `Right`, and `Delete` to the new palette helpers
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - document the new cursor/editing keys in the command-palette help overlay
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for cursor movement and forward delete

## Contracts
- `Left` moves the command-palette cursor left by one character boundary
- `Right` moves the command-palette cursor right by one character boundary
- `Delete` removes the character at the current cursor without moving the
  cursor past the deletion boundary
- Existing placeholder and shell-style editing flows continue to work

## Risks
- Character-boundary edits must stay UTF-8 safe
- Palette cursor and placeholder state must remain coherent after movement and
  delete operations
- Help text must stay aligned with the new key handling

## Verification Strategy
- Add failing tests first for left/right insertion positioning and forward
  delete
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
