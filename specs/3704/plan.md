# Plan: Issue #3704 - Preserve active placeholder focus across Ctrl+U/Ctrl+K in `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette line-clearing helpers so they special-case an active
scaffold placeholder before falling back to ordinary clear-to-start or
clear-to-end behavior. `Ctrl+U` should clear the prefix before the active
placeholder while keeping that placeholder focused, and `Ctrl+K` should clear
the suffix after the active placeholder while keeping it focused. Keep the
slice narrow so it layers cleanly on top of the existing placeholder movement
and deletion behavior.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update command-palette clear-to-start and clear-to-end helpers to preserve
    active placeholder focus when present
- `crates/tau-tui/src/interactive/app_commands.rs`
  - keep `Ctrl+U` and `Ctrl+K` routing on the updated palette helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for active-placeholder line-clearing focus behavior

## Contracts
- `Ctrl+U` from an active placeholder clears the prefix and keeps the
  placeholder focused
- `Ctrl+K` from an active placeholder clears the suffix and keeps the
  placeholder focused
- Ordinary non-placeholder line-clearing behavior remains unchanged
- Existing placeholder and editor flows continue to work

## Risks
- Placeholder span boundaries must stay coherent after line-clearing edits
- `Ctrl+U`/`Ctrl+K` must not regress ordinary non-placeholder behavior
- Focus preservation must cooperate with placeholder replacement logic

## Verification Strategy
- Add failing tests first for active-placeholder `Ctrl+U` and `Ctrl+K`
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
