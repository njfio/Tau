# Plan: Issue #3762 - Harness compact dashboard readability

## Cleanup Plan

1. Preserve the existing route, forms, IDs, and state-backed harness data.
2. Add compact desktop data markers for dashboard tables and proof metadata.
3. Widen the sidebar enough to avoid broken words while keeping the route inside the viewport.
4. Switch dashboard KPI cards to two columns at compact desktop widths.
5. Keep the active mission table semantic, but make it fixed-layout and hide low-priority columns after plan progress.
6. Stack proof metadata at compact desktop widths so run IDs and budget values remain readable.
7. Verify with targeted dashboard UI tests, full dashboard tests, static checks, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3762/`

## Risks / Mitigations

- Risk: hiding mission columns removes useful detail. Mitigation: only hide columns at compact desktop widths; wide layouts retain the full table.
- Risk: sidebar width reduces pane space. Mitigation: keep the panel width capped to the remaining viewport and use compact table behavior.
- Risk: proof header stacking adds vertical height. Mitigation: panes already have internal overflow bounds.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3762`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Visual fallback: bundled browser runtime captures `/tmp/tau-harness-continue-after.png` and confirms the page and dashboard table have no horizontal overflow at 1371px.
