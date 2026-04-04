# Plan: Issue #3706 - Make active scaffold placeholders atomic for `Ctrl+A`/`Ctrl+E` in `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette line-boundary helpers so they special-case an active
scaffold placeholder before falling back to ordinary absolute start/end
movement. When a placeholder is active, `Ctrl+A` should clear placeholder focus
and move the cursor to the placeholder start boundary, while `Ctrl+E` should
clear placeholder focus and move the cursor to the placeholder end boundary.
Once no placeholder is active, the existing absolute line-boundary behavior
should remain unchanged.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update command-palette start/end movement helpers to preserve placeholder
    boundary semantics when active
- `crates/tau-tui/src/interactive/app_commands.rs`
  - keep `Ctrl+A` and `Ctrl+E` routed to the updated helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for active-placeholder line-boundary escape behavior

## Contracts
- `Ctrl+A` from an active placeholder moves to that placeholder's start boundary
- `Ctrl+E` from an active placeholder moves to that placeholder's end boundary
- Ordinary non-placeholder `Ctrl+A`/`Ctrl+E` behavior remains unchanged
- Existing command-palette editor flows continue to work

## Risks
- Placeholder span boundaries must remain coherent after focus clears
- Start/end movement must not regress ordinary non-placeholder line navigation
- The new behavior should align with earlier placeholder-aware movement
  shortcuts instead of introducing a new editing model

## Verification Strategy
- Add failing tests first for active-placeholder `Ctrl+A` and `Ctrl+E`
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
