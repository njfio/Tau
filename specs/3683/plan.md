# Plan: Issue #3683 - Add command palette autocomplete and history to `tau-tui` REPL

Status: Implemented

## Approach
Promote the current command palette from a raw string input into a small command
launcher with a static command catalog, filtered suggestions, palette-local
history, and keyboard navigation. Keep the implementation inside `tau-tui`
without changing the existing command execution surface.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add palette selection/history state and helper methods
- `crates/tau-tui/src/interactive/app_keys.rs`
  - refine palette toggle behavior if needed
- `crates/tau-tui/src/interactive/app_commands.rs`
  - define the command catalog, suggestion matching, palette navigation, and
    execution behavior
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render palette suggestions with selection highlight and descriptions
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/green tests for palette rendering, autocomplete, and history recall

## Contracts
- `Ctrl+P` opens the palette and focuses command input
- `Up/Down` moves the active suggestion
- `Tab` replaces the current palette input with the active suggestion
- `Enter` executes the active suggestion if one exists, otherwise the typed
  command
- `Alt+Up/Down` recalls prior palette commands

## Risks
- Palette history and prompt history should not interfere with each other
- Suggestion selection should reset cleanly when the typed query changes
- Overlay height must expand enough to render useful suggestions without
  obscuring the rest of the UI too aggressively

## Verification Strategy
- Add failing tests first for suggestion render, autocomplete/execute, and
  history recall
- Re-run existing `interactive::app_gateway_tests`
- Build `tau-tui` after the scoped tests pass
