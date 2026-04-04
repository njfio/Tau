# Plan: Issue #3703 - Collapse scaffold separator whitespace when deleting active placeholders with Backspace/Delete in `tau-tui` command palette

Status: Reviewed

## Approach
Extend the active-placeholder branches in the command-palette character-deletion
helpers so they remove the placeholder span and normalize the adjacent
separator whitespace before returning. `Delete` and `Backspace` should keep the
remaining arguments single-spaced and avoid trailing blanks, while ordinary
non-placeholder character deletion remains unchanged. Keep the slice narrow so
it layers cleanly on top of the existing placeholder focus and earlier
word-deletion cleanup.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - update active-placeholder character-deletion branches to collapse redundant
    separator whitespace after removing the placeholder span
- `crates/tau-tui/src/interactive/app_commands.rs`
  - keep `Delete` and `Backspace` routing on the updated palette helpers
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN coverage for active-placeholder character-deletion whitespace
    cleanup

## Contracts
- `Delete` from an active placeholder removes the placeholder and leaves the
  remaining arguments single-spaced
- `Backspace` from an active placeholder removes the placeholder and trims
  redundant trailing separator whitespace
- Ordinary non-placeholder character-deletion behavior remains unchanged
- Existing placeholder and editor flows continue to work

## Risks
- Whitespace normalization must not remove real argument content
- First/last placeholder deletion must behave coherently with different
  placeholder positions
- Non-placeholder character deletion must not regress

## Verification Strategy
- Add failing tests first for active-placeholder `Delete` and `Backspace`
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Run `cargo clippy -p tau-tui --no-deps -- -D warnings`
- Build `tau-tui`
