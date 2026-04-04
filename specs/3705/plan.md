# Plan: Issue #3705 - Make `Esc` clear active placeholder focus before closing `tau-tui` command palette

Status: Reviewed

## Approach
Extend command-palette `Esc` handling so it checks for an active scaffold
placeholder before closing the palette. When placeholder focus is active, the
first `Esc` should clear that focus and keep the palette open. Once no
placeholder is active, `Esc` should continue to close the palette using the
existing close path. Keep the slice narrow so it layers on top of the current
placeholder editing behavior without changing unrelated input-mode semantics.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add or reuse a helper that clears active command-palette placeholder focus
- `crates/tau-tui/src/interactive/app_commands.rs`
  - route command-palette `Esc` through placeholder-aware handling before the
    ordinary close path
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for placeholder-aware `Esc` behavior

## Contracts
- `Esc` from an active placeholder clears placeholder focus and keeps the
  command palette open
- `Esc` with no active placeholder closes the command palette
- Existing command-palette close behavior remains intact once placeholder focus
  is gone
- Existing command-palette editor flows continue to work

## Risks
- Placeholder focus state must clear cleanly without mutating scaffold text
- `Esc` routing must not interfere with non-placeholder close behavior
- Command-palette overlay feedback must stay coherent after focus clears

## Verification Strategy
- Add failing tests first for active-placeholder `Esc` focus clearing and the
  subsequent palette close path
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
