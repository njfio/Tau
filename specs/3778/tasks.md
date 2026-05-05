# Tasks 3778: Keep Proposal Safety Summary Visible

- [x] T1: Add a conformance test proving proposal safety summary ordering and
  compact first-viewport markers.
- [x] T2: Reorder proposal detail rows and add compact-scroll sizing.
- [x] T3: Run focused dashboard tests, full dashboard tests, lint/style checks,
  and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3778` failed because
  proposal detail did not prioritize the safety summary before explanatory rows.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3778` passed.
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_377` passed
  (9 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (161 tests).
- INTEGRATION: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- FORMAT: `cargo fmt --check -p tau-dashboard-ui` passed.
- LINT: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
  passed.
- STATIC: `git diff --check` passed.
- BROWSER: Browser Use `iab` was attempted and unavailable; fallback Playwright
  loaded `file:///tmp/tau-harness-after.html`, wrote
  `/tmp/tau-harness-continue18-after.png`, reported zero console errors, and
  confirmed proposal detail, benchmark panel, and TUI companion are within the
  visible viewport.
