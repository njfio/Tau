# Tasks 3779: Keep Recent Audit Proof Visible

- [x] T1: Add a conformance test proving audit first-viewport markers and
  compact audit column priority.
- [x] T2: Add scoped review-density and audit-table compacting.
- [x] T3: Run focused dashboard tests, full dashboard tests, lint/style checks,
  and browser geometry validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3779` failed because
  first-viewport review/audit markers were missing.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3779` passed.
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_377` passed
  (10 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (162 tests).
- INTEGRATION: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- FORMAT: `cargo fmt --check -p tau-dashboard-ui` passed.
- LINT: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
  passed.
- STATIC: `git diff --check` passed.
- BROWSER: Browser Use `iab` was attempted and unavailable; fallback Playwright
  loaded `file:///tmp/tau-harness-after.html`, wrote
  `/tmp/tau-harness-continue19-after.png`, reported zero console errors, and
  confirmed actions, policy, proposal detail, recent audit proof, benchmark
  panel, memory/artifacts, and TUI companion are inside the visible viewport.
