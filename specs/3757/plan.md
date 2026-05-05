# Plan: Issue #3757 - State-backed harness benchmark and audit panels

## Approach

1. Introduce `TauOpsDashboardHarnessSnapshot` with benchmark rows and recent
   audit rows.
2. Render benchmark and audit sections from the snapshot, keeping the current
   deterministic defaults.
3. Add gateway collection code that reads:
   - `ops-harness/m334/latest.json`
   - `ops-harness/audit.jsonl`
4. Populate the shell context from gateway state before rendering `/ops/harness`.
5. Add tests for direct dashboard rendering and gateway state-backed rendering.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests/ops_config_training_safety.rs`
- `specs/3757/`

## Interfaces

- Dashboard context: `TauOpsDashboardHarnessSnapshot`
- Benchmark state: `<gateway-state-dir>/ops-harness/m334/latest.json`
- Audit state: `<gateway-state-dir>/ops-harness/audit.jsonl`
- Marker: `data-proof-source="state|fallback"`
- Marker: `data-audit-source="state|fallback"`

## Risks / Mitigations

- Risk: malformed proof/audit files break the page.
  Mitigation: parse defensively and fall back to deterministic defaults.
- Risk: static fixture values keep masquerading as live state.
  Mitigation: expose explicit source markers for proof and audit sections.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3757`
- RED/GREEN: `cargo test -p tau-gateway integration_spec_3757`
- Regression: `cargo test -p tau-dashboard-ui`
- Regression: `cargo test -p tau-gateway`
- Static: `cargo fmt --check -p tau-dashboard-ui -p tau-gateway`
- Static: `cargo clippy -p tau-dashboard-ui -p tau-gateway --all-targets -- -D warnings`
