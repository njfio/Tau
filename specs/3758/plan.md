# Plan: Issue #3758 - Mission harness operator UI usability pass

## Cleanup Plan

1. Keep behavior and backend state untouched.
2. Preserve all existing IDs, routes, forms, and data markers.
3. Replace the thin harness style block with a stronger operator-console design system scoped to `#tau-ops-harness-panel`.
4. Add minimal semantic wrapper classes only where needed for visual hierarchy and testable contracts.
5. Add regression tests for the visual/interaction contract.
6. Verify with dashboard tests, gateway harness tests, fmt, clippy, and a browser screenshot smoke.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3758/`

## Risks / Mitigations

- Risk: visual changes break existing marker tests. Mitigation: preserve all existing ids/data attributes and run the full dashboard UI suite.
- Risk: CSS becomes another large opaque block. Mitigation: keep selectors scoped and contract markers explicit.
- Risk: browser rendering differs from string tests. Mitigation: generate standalone SSR HTML and validate with Playwright screenshot checks.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3758`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3756 functional_spec_3757`
- Regression: `cargo test -p tau-dashboard-ui`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Visual: generated SSR HTML inspected with Playwright screenshot/non-overlap checks
