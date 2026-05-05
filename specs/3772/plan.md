# Plan: Issue #3772 - Harness compact mission state visibility

## Cleanup Plan

1. Preserve existing mission row data and compact hidden-column behavior.
2. Add a focused regression test for inline mission state and gate chips in the goal cell.
3. Update mission row markup to include compact title and metadata spans.
4. Add scoped CSS for compact mission metadata chips.
5. Verify with targeted/full dashboard tests, gateway integration, static checks, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3772/`

## Risks / Mitigations

- Risk: duplicated gate state could feel noisy when the full verification column is visible. Mitigation: compact metadata is small and row-local, and the existing verification column remains authoritative.
- Risk: inline chips could widen the first column. Mitigation: chips wrap inside the goal cell and use compact padding/font markers.
- Risk: status chip color inheritance could make state chips ambiguous. Mitigation: add explicit mission state and gate chip markers.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3772`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3762`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: render the post-change preview and confirm mission row chips are visible without horizontal overflow.
