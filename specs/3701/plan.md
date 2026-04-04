# Plan: Issue #3701 - Add placeholder-aware Alt+B/Alt+F word escape to `tau-tui` command palette

Status: Reviewed

## Approach
Extend the command-palette word-movement helpers so they special-case an active
scaffold placeholder before falling back to ordinary word-wise movement.
`Alt+B` should clear placeholder focus and keep the cursor at the active
placeholder start boundary, while `Alt+F` should clear placeholder focus and
jump to the active placeholder end boundary. Keep the slice narrow so it layers
cleanly on top of the existing placeholder autofocus, cycling, and
character-wise escape behavior.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update command-palette word-movement helpers to recognize an active
    placeholder span before ordinary word-wise movement
- `crates/tau-tui/src/interactive/app_commands.rs`
  - keep `Alt+B`/`Alt+F` routing on the updated palette helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for placeholder-aware word escape behavior

## Contracts
- `Alt+B` from an active placeholder clears placeholder focus and stays at that
  placeholder start boundary
- `Alt+F` from an active placeholder clears placeholder focus and moves to that
  placeholder end boundary
- Ordinary word-wise movement remains unchanged when no placeholder is active
- Existing placeholder and editor flows continue to work

## Risks
- Word-wise movement must not jump into adjacent placeholders after the change
- Placeholder span boundaries must remain coherent after word escape
- Non-placeholder word movement must not regress

## Verification Strategy
- Add failing tests first for active-placeholder `Alt+B` and `Alt+F`
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
