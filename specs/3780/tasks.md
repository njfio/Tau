# Tasks 3780: Keep All Verification Gates Visible

- [x] T1: Add a conformance test proving all-gate visibility markers and gate
  ordering.
- [x] T2: Add compact all-gates-first-viewport verification gate layout.
- [x] T3: Run focused dashboard tests, full dashboard tests, lint/style checks,
  and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3780` failed because
  all-gates-first-viewport markers were missing.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3780` passed.
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_377` plus
  `cargo test -p tau-dashboard-ui functional_spec_3780` passed.
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (163 tests).
- INTEGRATION: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- FORMAT: `cargo fmt --check -p tau-dashboard-ui` passed.
- LINT: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
  passed.
- STATIC: `git diff --check` passed.
- BROWSER: Browser Use `iab` was attempted and unavailable; fallback Playwright
  loaded `file:///tmp/tau-harness-after.html`, wrote
  `/tmp/tau-harness-continue20-after.png`, reported zero console errors, and
  confirmed all verification-gate chips fit inside the gate card while memory,
  artifacts, audit, benchmark, and TUI companion remain visible.
