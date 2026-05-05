# Tasks: Issue #3759 - Harness static preview action guard

Status: Implemented

- [x] T1 (SPEC): create spec/plan/tasks for static preview action behavior.
- [x] T2 (RED): add dashboard UI regression tests for the preview guard contract.
- [x] T3 (GREEN): add a file-protocol-only harness preview guard.
- [x] T4 (VERIFY): run targeted/full tests, static checks, and Browser Use click validation.

## Verification Evidence

- RED: `cargo test -p tau-dashboard-ui functional_spec_3759` failed because
  the harness route did not expose `tau-ops-harness-preview-status` or the
  file-protocol preview guard markers.
- GREEN: `cargo test -p tau-dashboard-ui functional_spec_3759` passed
  (1 test).
- REGRESSION: `cargo test -p tau-dashboard-ui functional_spec_375`
  passed (6 tests).
- REGRESSION: `cargo test -p tau-dashboard-ui` passed (142 tests).
- REGRESSION: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
  passed (1 test).
- STATIC: `cargo fmt --check -p tau-dashboard-ui` passed.
- STATIC: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
  passed.
- INTERACTION: Browser Use loaded `file:///tmp/tau-harness-after.html`, clicked
  `#tau-ops-harness-run-benchmark`, and confirmed the URL stayed on the local
  preview while the form preserved `action="/ops/harness/run-benchmark"`,
  `method="post"`, and received `data-preview-submit-blocked="true"`.
- INTERACTION: Browser Use console log check returned zero entries after the
  guarded click.
