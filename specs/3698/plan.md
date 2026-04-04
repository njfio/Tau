# Plan: Issue #3698 - Add forward word deletion shortcut to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette editor with a forward word-deletion helper that
mirrors the palette's existing word-boundary movement logic, then route
`Alt+D` through the existing command-palette key handler. Keep the slice
limited to forward word deletion so it layers cleanly on top of line movement,
word movement, and backward word deletion.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add a command-palette helper for deleting the next word relative to the
    current cursor
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `Alt+D` to the new command-palette helper
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - document `Alt+D` in the command-palette help overlay
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for line-start and mid-line next-word deletion

## Contracts
- `Alt+D` deletes the next word segment relative to the command-palette cursor
- Deletion at line start leaves the cursor at the same replacement boundary
- Mid-line deletion preserves following words and cursor coherence
- Existing placeholder and shell-style editing flows continue to work

## Risks
- Forward word-deletion semantics must stay aligned with palette word movement
- Palette cursor and placeholder state must remain coherent after deletion
- Help text must stay aligned with the new key handling

## Verification Strategy
- Add failing tests first for line-start and mid-line `Alt+D` deletion
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
