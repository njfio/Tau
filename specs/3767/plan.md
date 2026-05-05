# Plan: Issue #3767 - Harness proof evidence first-screen priority

## Cleanup Plan

1. Preserve existing tool evidence rows, IDs, data attributes, and compact table CSS.
2. Add a focused regression test for proof evidence placement.
3. Move the tool evidence section into the primary proof grid before secondary proof detail sections.
4. Add a route-visible placement marker and grid-span styling.
5. Verify with red/green test evidence, harness regression tests, full dashboard UI tests, static checks, gateway integration, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3767/`

## Risks / Mitigations

- Risk: moving the table could make acceptance criteria less prominent. Mitigation: keep acceptance directly below tool evidence and preserve all current proof detail sections.
- Risk: spanning evidence across the grid could increase vertical scroll. Mitigation: the proof pane already scrolls and the table remains compact at desktop width.
- Risk: existing evidence table tests could become brittle. Mitigation: preserve current markers and add a placement-specific test.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3767`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: bundled browser runtime captures the post-change preview and confirms the evidence section is visible inside the proof viewport at 1371px.
