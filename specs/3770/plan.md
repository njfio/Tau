# Plan: Issue #3770 - Harness operator log first-screen placement

## Cleanup Plan

1. Preserve the existing operator log content and wrapping behavior.
2. Add a focused regression test for proof DOM order and first-screen log markers.
3. Move the operator log into the proof grid immediately after tool evidence.
4. Add compact log height styling so the proof pane remains scannable.
5. Verify with targeted/full dashboard tests, gateway harness integration, static checks, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3770/`

## Risks / Mitigations

- Risk: moving the log could hide secondary acceptance/gate detail. Mitigation: cap log pre height and keep secondary sections directly after it.
- Risk: proof evidence first-screen priority could regress. Mitigation: assert tool evidence still precedes the log and all secondary detail.
- Risk: DOM order change could affect existing harness tests. Mitigation: rerun the full `functional_spec_376` set and full dashboard UI test package.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3770`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_376`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: render the post-change preview and confirm operator log first-screen geometry.
