# Plan: Issue #3763 - Harness proof evidence and log wrapping

## Cleanup Plan

1. Preserve existing proof/log markup, route actions, forms, and IDs.
2. Add compact proof-evidence and log-wrap data markers.
3. Make the tool evidence table fixed-layout at compact desktop widths.
4. Hide the low-value artifact column only in compact proof evidence view.
5. Wrap tool table cells and preformatted log/TUI text inside their panes.
6. Verify with red/green test evidence, harness regression tests, full dashboard UI tests, static checks, gateway integration, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3763/`

## Risks / Mitigations

- Risk: hiding the artifact column removes a useful direct path. Mitigation: only hide it at compact width; the wide table still keeps all columns.
- Risk: `pre-wrap` changes log density. Mitigation: apply only to harness operator log and TUI companion at compact desktop width.
- Risk: proof table wrapping makes rows taller. Mitigation: proof pane already has internal vertical scrolling.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3763`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: bundled browser runtime captures `/tmp/tau-harness-continue3-after.png` and confirms proof/log/TUI content no longer overflows horizontally at 1371px.
