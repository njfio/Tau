# Plan: Issue #3765 - Harness compact evidence column priority

## Cleanup Plan

1. Preserve tool evidence rows, statuses, and wide-screen table semantics.
2. Add a focused regression test for compact evidence column priority.
3. Hide call ID in addition to artifact at compact desktop width.
4. Give runtime and status fixed nowrap compact widths.
5. Verify with red/green test evidence, harness regression tests, full dashboard UI tests, static checks, gateway integration, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3765/`

## Risks / Mitigations

- Risk: hiding call IDs reduces compact diagnostic detail. Mitigation: only hide them at compact width; the full table keeps identifiers.
- Risk: fixed widths could crowd plan node text. Mitigation: plan node remains allowed to wrap while runtime/status stay readable.
- Risk: overlapping with the prior evidence wrapping contract. Mitigation: update the compact proof tests to assert the new column-priority behavior directly.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3765`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: bundled browser runtime captures the post-change preview and confirms compact evidence table readability at 1371px.
