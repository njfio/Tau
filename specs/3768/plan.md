# Plan: Issue #3768 - Harness proof DAG compact single row

## Cleanup Plan

1. Preserve the existing proof DAG node IDs, labels, statuses, and current-node metadata.
2. Add a focused regression test for compact single-row DAG styling and semantics.
3. Change desktop DAG styling from auto-fit wide pills to five compact equal columns.
4. Keep narrow responsive collapse behavior unchanged.
5. Verify with red/green test evidence, harness regression tests, full dashboard UI tests, static checks, gateway integration, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3768/`

## Risks / Mitigations

- Risk: compact labels could truncate useful node names. Mitigation: keep full text in the DOM and only reduce spacing/font size enough to avoid wrapping.
- Risk: desktop layout change could affect mobile. Mitigation: preserve the existing max-width responsive DAG collapse.
- Risk: visual changes could reintroduce proof evidence overflow. Mitigation: rerun browser geometry for DAG row shape and evidence table width.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3768`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: bundled browser runtime captures the post-change preview and confirms DAG nodes share one row at 1371px.
