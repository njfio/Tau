# Plan: Issue #3764 - Harness self-improvement action priority

## Cleanup Plan

1. Preserve existing proposal detail, policy, audit, routes, forms, IDs, and approval gating.
2. Add a focused regression test for action placement and compact action styling.
3. Move `Operator Actions` before the conservative policy section.
4. Add durable action-priority markers and compact grid styling for the controls.
5. Verify with red/green test evidence, harness regression tests, full dashboard UI tests, static checks, gateway integration, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3764/`

## Risks / Mitigations

- Risk: moving the policy below actions may make the safety context less prominent. Mitigation: keep the policy immediately after actions and preserve all safety markers.
- Risk: action-grid styling could shrink button labels. Mitigation: use two equal columns and make apply span the full row.
- Risk: route forms could regress during markup movement. Mitigation: keep existing IDs/actions/methods and assert them in the regression test.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3764`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: bundled browser runtime captures the post-change preview and confirms the action section is above the policy/audit context without horizontal document overflow at 1371px.
