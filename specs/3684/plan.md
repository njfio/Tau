# Plan: Issue #3684 - Add command palette aliases and richer feedback to `tau-tui` REPL

Status: Implemented

## Approach
Extend the command catalog with explicit aliases, teach palette matching and
execution to resolve canonical commands from either command names or aliases,
and improve the overlay so it shows alias metadata, current match counts, and
an explicit no-match state. Keep the change inside `tau-tui` and avoid changing
the command execution surface outside the palette.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - keep palette state helpers compatible with alias-aware selection
- `crates/tau-tui/src/interactive/app_commands.rs`
  - add aliases to the command catalog
  - resolve canonical commands from aliases
  - preserve sensible history/selection behavior
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render alias metadata and palette feedback lines
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for alias execution and feedback rendering

## Contracts
- Palette suggestions render canonical command names and configured aliases
- `Enter` on an exact alias executes the corresponding canonical command
- Palette feedback includes match count and selected command context
- When there are zero matches, the overlay renders an explicit no-match hint

## Risks
- Alias collisions could produce ambiguous execution
- History should remain predictable when an alias resolves to a canonical command
- Feedback text should fit in the current overlay without making it noisy

## Verification Strategy
- Add failing tests first for alias render/execution and no-match feedback
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
