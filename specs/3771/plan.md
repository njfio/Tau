# Plan: Issue #3771 - Harness verification gates first-screen priority

## Cleanup Plan

1. Preserve all verification gate IDs, statuses, labels, and counts.
2. Add a focused regression test for proof grid ordering and compact secondary chip markers.
3. Reorder secondary proof sections so gates follow acceptance and precede memory/artifacts.
4. Add compact acceptance/gate chip spacing without changing global chip semantics.
5. Verify with targeted/full dashboard tests, gateway harness integration, static checks, and browser geometry.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/3771/`

## Risks / Mitigations

- Risk: moving gates could make memory summary less visible. Mitigation: memory remains immediately after gates and retains its marker contract.
- Risk: compact chip spacing could hurt readability. Mitigation: change only acceptance/gate padding and gap by a small amount.
- Risk: proof grid priority markers could drift. Mitigation: update existing proof-priority tests and add a dedicated gate-priority test.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3771`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_3770`
- Regression: `cargo test -p tau-dashboard-ui functional_spec_37`
- Regression: `cargo test -p tau-dashboard-ui`
- Integration: `cargo test -p tau-gateway integration_spec_3757_c03_ops_harness_route_reflects_state_backed_proof_and_audit`
- Static: `cargo fmt --check -p tau-dashboard-ui`
- Static: `cargo clippy -p tau-dashboard-ui --all-targets -- -D warnings`
- Browser fallback: render the post-change preview and confirm gate first-screen geometry.
