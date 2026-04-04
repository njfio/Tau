# Plan: Issue #3697 - Add word-wise deletion shortcut to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette editor with a backward word-deletion helper that
mirrors the palette's existing word-boundary movement logic, then route
`Ctrl+W` through the existing command-palette key handler. Keep the slice
limited to backward word deletion so it layers cleanly on top of the line-wise
editing from `#3695` and word-wise movement from `#3696`.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add a command-palette helper for deleting the previous word relative to the
    current cursor
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `Ctrl+W` to the new command-palette helper
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - document `Ctrl+W` in the command-palette help overlay
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for end-of-line and mid-line previous-word deletion

## Contracts
- `Ctrl+W` deletes the previous word segment relative to the command-palette
  cursor
- Deletion at line end leaves the cursor at the replacement boundary
- Mid-line deletion preserves the following word and cursor coherence
- Existing placeholder and shell-style editing flows continue to work

## Risks
- Backward word-deletion semantics must stay aligned with palette word movement
- Palette cursor and placeholder state must remain coherent after deletion
- Help text must stay aligned with the new key handling

## Verification Strategy
- Add failing tests first for end-of-line and mid-line `Ctrl+W` deletion
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
