# Plan: Issue #3761 - Harness desktop preview layout density

## Cleanup Plan

1. Preserve existing harness markup, form routes, and preview guards.
2. Add a testable desktop layout marker to the harness panel.
3. Keep desktop columns as zero-min fractional tracks so the grid fits the visible shell while table content scrolls inside panes.
4. Move the single-column media query from the overly broad breakpoint to a narrow breakpoint.
5. Add viewport-height bounds and internal overflow to the tall harness windows.
6. Add pane-local wrapping for long labels, definition-list values, and action controls.
7. Verify with targeted UI tests, full dashboard UI tests, static checks, Browser Use DOM validation, and screenshot fallback geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3761/`

## Risks / Mitigations

- Risk: three columns become cramped on narrow screens. Mitigation: use zero-min tracks, internal table scrolling, and collapse at the narrow breakpoint.
- Risk: hidden content becomes inaccessible. Mitigation: use internal `overflow: auto`, not clipping.
- Risk: long proposal labels widen or clip the review pane. Mitigation: wrap text and constrain controls inside pane bounds.
- Risk: changing layout breaks existing preview guards. Mitigation: rerun the 3759/3760 filtered tests.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3761`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Visual: Browser Use opens rendered `file://` HTML and confirms dashboard, proof, review, TUI, and operator actions are present with zero console logs.
- Visual fallback: bundled browser runtime captures `/tmp/tau-harness-after-1371.png` and confirms `scrollWidth == clientWidth == 1371`, with the right review/TUI edge at 1344px.
