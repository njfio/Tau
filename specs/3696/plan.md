# Plan: Issue #3696 - Add word-wise movement shortcuts to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette input model with word-wise cursor movement helpers
that mirror the main input editor's behavior, then route `Alt+B` and `Alt+F`
through the existing palette key handler. Keep the scope to movement only so it
layers cleanly on top of the line-wise editing added in `#3695`.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add command-palette helpers for backward and forward word movement
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route `Alt+B` and `Alt+F` to the new palette helpers
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - document word-wise palette movement in the help overlay
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for word-wise command-palette movement

## Contracts
- `Alt+B` moves the command-palette cursor to the previous word boundary
- `Alt+F` moves the command-palette cursor to the next word boundary
- Existing line-wise editing and placeholder flows continue to work

## Risks
- Word-boundary semantics need to stay aligned with the main input editor
- Palette cursor state must remain coherent when switching between word-wise
  movement and placeholder-focused editing
- Help text must stay aligned with the new key handling

## Verification Strategy
- Add failing tests first for backward and forward word movement
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
