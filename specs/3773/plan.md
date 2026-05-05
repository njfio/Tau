# Plan: Issue #3773 - Harness TUI companion first-viewport fit

## Cleanup Plan

1. Preserve existing TUI summary content and command markers.
2. Add a focused regression test for TUI first-viewport priority and compact right-column bounds.
3. Tighten review/TUI max heights and set bounded box sizing for harness windows.
4. Compact only the TUI companion pre block.
5. Verify with targeted/full dashboard tests, gateway integration, static checks, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3773/`

## Risks / Mitigations

- Risk: review detail gets less vertical room. Mitigation: review pane already scrolls internally and keeps action priority before policy/audit.
- Risk: terminal output becomes too dense. Mitigation: compact only the TUI companion pre block and preserve wrapping.
- Risk: changing window box sizing affects max-height calculations. Mitigation: rerun harness layout and right-column tests plus browser geometry.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3773`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3761`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3764`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: render the post-change preview and confirm the TUI companion bottom is inside the desktop viewport.
