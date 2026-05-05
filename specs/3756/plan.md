# Plan: Issue #3756 - Mission harness operator UI and TUI proof view

## Approach

1. Add `TauOpsDashboardRoute::Harness` and expose `/ops/harness` from the
   dashboard shell route model.
2. Add a deterministic mission harness panel that follows the provided template:
   dashboard window, mission proof window, self-improvement review/apply window,
   and TUI companion.
3. Wire `/ops/harness` through the OpenResponses gateway router with the same
   shell controls as other ops routes.
4. Add deterministic harness action routes for benchmark proof generation,
   proposal audit actions, and proposal diff review.
5. Add CSS-only shell presentation so `/ops/harness` opens directly into the
   supplied dark operator workspace rather than exposing inactive route panels.
6. Extend `tau-tui` with an optional harness summary panel for benchmark proof
   status.
7. Verify with dashboard UI tests, gateway route tests, TUI renderer tests,
   live browser/HTTP smoke checks, formatting, clippy, and scoped cargo tests.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/endpoints.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_handlers.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_bootstrap.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests/ops_auth_navigation.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests/ops_config_training_safety.rs`
- `crates/tau-tui/src/lib.rs`
- `crates/tau-tui/src/main.rs`
- `specs/3756/`

## Interfaces

- Dashboard route: `/ops/harness`
- Benchmark action: `POST /ops/harness/run-benchmark`
- Proposal actions: `POST /ops/harness/proposals/{proposal_id}/{action}`
- Proposal diff review: `GET /ops/harness/proposals/{proposal_id}/diff`
- Benchmark form marker: `data-command="tau_agent_harness"`
- Self-improvement policy marker:
  `data-allowed-targets="skill,config,prompt"` and
  `data-blocked-targets="source-code,safety-policy"`
- TUI harness summary: `OperatorHarnessSummary`

## Risks / Mitigations

- Risk: static proof data could be mistaken for live execution.
  Mitigation: execute only the deterministic M334 harness and write an explicit
  proof artifact under gateway state.
- Risk: UI apply controls imply autonomous mutation.
  Mitigation: mark apply as disabled, approval-required, and policy-gated;
  reject direct apply POSTs with an audited approval-required result.
- Risk: dashboard route drift.
  Mitigation: cover route model, gateway route, and hidden-panel regression.

## Verification

- RED/GREEN: `cargo test -p tau-dashboard-ui functional_spec_3756`
- GREEN: `cargo test -p tau-tui functional_operator_shell_renderer_includes_harness_summary`
- GREEN: `cargo test -p tau-gateway integration_spec_3756_c04`
- GREEN: `cargo test -p tau-gateway integration_spec_3756_c05`
- Static: `cargo fmt --check -p tau-dashboard-ui -p tau-gateway -p tau-tui`
- Static: `cargo clippy -p tau-dashboard-ui -p tau-gateway -p tau-tui --all-targets -- -D warnings`
- Live: Chrome smoke for `http://127.0.0.1:18787/ops/harness`
- Live: gateway HTTP smoke for benchmark proof, proposal diff, and blocked apply
- Regression: `cargo test -p tau-dashboard-ui`
- Regression: `cargo test -p tau-tui`
- Regression: `cargo test -p tau-gateway`
