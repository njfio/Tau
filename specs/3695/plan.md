# Plan: Issue #3695 - Add shell-style cursor and kill shortcuts to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette input state with simple cursor movement and
line-clearing helpers, then route shell-style control shortcuts through the
existing palette key handler. Keep the scope narrow to line-wise editing so the
new shortcuts fit the current palette model without introducing a new text
editor abstraction.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add command-palette helpers for moving to line start/end and clearing to
    line start/end
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `Ctrl+A`, `Ctrl+E`, `Ctrl+U`, and `Ctrl+K` to the new palette helpers
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - document the new command-palette editing shortcuts in the help overlay
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for shell-style palette editing behavior

## Contracts
- `Ctrl+A` moves the command-palette cursor to the start of the line
- `Ctrl+E` moves the command-palette cursor to the end of the line
- `Ctrl+U` clears from the current cursor position to the line start
- `Ctrl+K` clears from the current cursor position to the line end

## Risks
- Palette cursor state must remain coherent with autocomplete and placeholder
  editing paths
- Clear operations should not accidentally reset palette selection or history
  state in a way that surprises operators
- Help text needs to stay aligned with the actual palette key handling

## Verification Strategy
- Add failing tests first for line movement and line-clearing behavior
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
