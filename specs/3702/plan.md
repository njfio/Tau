# Plan: Issue #3702 - Collapse scaffold separator whitespace when deleting active placeholders in `tau-tui` command palette

Status: Reviewed

## Approach
Extend the active-placeholder branches in the command-palette word-deletion
helpers so they remove the placeholder span and normalize the adjacent separator
whitespace before returning. `Alt+D` and `Ctrl+W` should keep the remaining
arguments single-spaced and avoid trailing blanks, while ordinary non-placeholder
word deletion remains unchanged. Keep the slice narrow so it layers cleanly on
top of the existing placeholder focus and word-deletion behavior.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update active-placeholder word-deletion branches to collapse redundant
    separator whitespace after removing the placeholder span
- `crates/tau-tui/src/interactive/app_commands.rs`
  - keep `Alt+D` and `Ctrl+W` routing on the updated palette helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for active-placeholder deletion whitespace cleanup

## Contracts
- `Alt+D` from an active placeholder removes the placeholder and leaves the
  remaining arguments single-spaced
- `Ctrl+W` from an active placeholder removes the placeholder and trims
  redundant trailing separator whitespace
- Ordinary non-placeholder word-deletion behavior remains unchanged
- Existing placeholder and editor flows continue to work

## Risks
- Whitespace normalization must not eat real argument content
- First/last placeholder deletion must behave coherently with different cursor
  positions
- Non-placeholder word deletion must not regress

## Verification Strategy
- Add failing tests first for active-placeholder `Alt+D` and `Ctrl+W`
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
