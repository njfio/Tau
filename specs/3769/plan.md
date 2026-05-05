# Plan: Issue #3769 - Harness operator action tones

## Cleanup Plan

1. Preserve existing form actions, methods, IDs, and preview guard behavior.
2. Add a focused regression test for action-tone markers and scoped action styling.
3. Replace the broad submit-button green rule with a neutral default and explicit approval, reject, benchmark, dry-run, disabled tones.
4. Verify with targeted harness tests, static checks, gateway route integration, and browser preview evidence.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3769/`

## Risks / Mitigations

- Risk: changing the broad submit rule could make a primary action look too weak. Mitigation: explicitly style `Approve` and benchmark/dry-run tones.
- Risk: action markers could drift from form behavior. Mitigation: keep the existing preview guard/action-form test green.
- Risk: visual tone changes could affect unrelated dashboard buttons. Mitigation: scope all changes to `#tau-ops-harness-panel`.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3769`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3759_c02_c03_harness_static_preview_guard_preserves_gateway_forms`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_376`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: render the post-change preview and inspect action button tones.
