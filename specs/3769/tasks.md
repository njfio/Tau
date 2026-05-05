# Tasks: Issue #3769 - Harness operator action tones

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for scoped operator action tones.
- [x] T2 (RED): add dashboard UI regression test for durable action-tone markers.
- [x] T3 (GREEN): scope harness action CSS and add action-tone markers.
- [x] T4 (VERIFY): run targeted/full tests, static checks, gateway integration, and browser preview validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3769` failed before implementation on the missing `data-action-tone="benchmark"` marker.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3769` passed after action-tone markers and scoped CSS were added.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3759_c02_c03_harness_static_preview_guard_preserves_gateway_forms` passed.
- Regression: `cargo test -p tau-dashboard-ui functional_spec_376` passed with 10 harness UI tests.
- Regression: `cargo test -p tau-dashboard-ui` passed with 152 tests.
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit` passed.
- Static: `cargo fmt --check -p tau-dashboard-ui` passed.
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings` passed.
- Static: `git diff --check` passed.
- Browser fallback: `/tmp/tau-harness-continue9-after.png` captured the rendered preview; computed styles showed benchmark and dry-run using blue secondary gradients, approve using a green approval gradient, reject using a red destructive gradient, apply remaining muted/disabled, document `clientWidth=1371`, `scrollWidth=1371`, panel `width=1263`, panel `scrollWidth=1261`, and no console errors.
- Browser Use caveat: in-app Browser attach was attempted through the required `iab` runtime, but the runtime did not attach cleanly in this thread, so visual verification used the bundled-browser fallback.
