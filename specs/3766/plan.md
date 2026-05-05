# Plan: Issue #3766 - Harness compact navigation rail

## Cleanup Plan

1. Preserve existing sidebar links, route IDs, hrefs, and non-harness labels.
2. Add short rail-label data attributes to the existing nav anchors.
3. Add harness-route CSS that displays compact pseudo labels and narrows the rail.
4. Adjust the harness panel width budget to reclaim the rail space.
5. Verify with red/green test evidence, harness regression tests, full dashboard UI tests, static checks, gateway integration, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3766/`

## Risks / Mitigations

- Risk: pseudo-label styling could hide accessible link text. Mitigation: keep the full anchor text in the DOM and expose short labels only visually through route-scoped CSS.
- Risk: non-harness routes could inherit compact labels. Mitigation: scope all compact rail styling to `#tau-ops-shell[data-active-route="harness"]`.
- Risk: width-budget changes could reintroduce horizontal overflow. Mitigation: verify rendered geometry at desktop preview width.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3766`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: bundled browser runtime captures the post-change preview and confirms no horizontal overflow at 1371px.
