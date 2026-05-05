# Tasks: Issue #3760 - Ops shell static preview navigation guard

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for static preview link navigation behavior.
- [x] T2 (RED): add dashboard UI regression tests for the preview link guard contract.
- [x] T3 (GREEN): add a file-protocol-only ops shell link guard.
- [x] T4 (VERIFY): run targeted/full tests, static checks, and Browser Use click validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3760` failed because
  the shell did not expose `tau-ops-static-preview-status` or the
  file-protocol route-link guard markers.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3760` passed
  (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_37` passed
  (7 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (143 tests).
- REGRESSION: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed (1 test).
- STATIC: `cargo fmt --check -p tau-dashboard-ui` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
  passed.
- INTERACTION: Browser Use loaded `file:///tmp/tau-harness-after.html`, clicked
  `#tau-ops-nav-agent-fleet a`, and confirmed the URL stayed on the local
  preview while the link preserved `href="/ops/agents"` and received
  `data-preview-link-blocked="true"`.
- INTERACTION: Browser Use console log check returned zero entries after the
  guarded click.
