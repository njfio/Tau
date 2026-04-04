# Plan: Issue #3685 - Add command palette paging and full browsing to `tau-tui` REPL

Status: Implemented

## Approach
Keep the command palette selection indexed across the full match list, then
compute a render window anchored around the selected index so the highlight
stays visible while the operator moves. Add boundary and page navigation in the
palette key handler and surface the visible range in the overlay footer.

## Affected Modules
- `crates/tau-tui/src/interactive/app.rs`
  - add palette helpers for first/last/page navigation and visible window math
- `crates/tau-tui/src/interactive/app_commands.rs`
  - wire stronger command palette navigation keys
- `crates/tau-tui/src/interactive/ui_overlays.rs`
  - render a selected window instead of always the first few matches
  - show visible-range feedback in the palette footer
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for deeper browsing and paging controls

## Contracts
- `Up/Down` continues to move a single selection across the full match list
- `PageUp`/`PageDown` move by the visible page size
- `Home`/`End` jump to the first/last result
- Rendered suggestions always include the active selection if matches exist
- Footer feedback shows the visible range and total count

## Risks
- Selection reset behavior on query change must remain intuitive
- Window math can drift off-by-one near the start/end of the result set
- Overlay height and visible page size must stay aligned

## Verification Strategy
- Add failing tests first for deeper browsing render and paging keys
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
