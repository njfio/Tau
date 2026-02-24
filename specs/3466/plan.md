# Plan: Issue #3466 - M302 fail-closed ops control-action outcomes

Status: Implemented

## Approach
1. Add RED integration tests in
   `crates/tau-gateway/src/gateway_openresponses/tests.rs` for:
   - missing action form submit -> deterministic redirect markers,
   - invalid action form submit -> deterministic redirect markers,
   - valid action submit -> deterministic applied marker.
2. Add RED UI contract test(s) in `crates/tau-dashboard-ui/src/tests.rs` for a
   control-action outcome panel marker contract.
3. Extend `OpsShellControlsQuery` with normalized parsing for:
   - `control_action_status`,
   - `control_action`,
   - `control_action_reason`.
4. Extend command-center shell context wiring to carry normalized marker values
   and render a deterministic status panel in `tau-dashboard-ui`.
5. Update `handle_ops_dashboard_control_action` to fail closed:
   - missing action -> redirect marker (`missing`),
   - invalid/apply error -> redirect marker (`failed`),
   - success -> redirect marker (`applied`).
6. Run scoped tests and quality gates, then update spec/tasks evidence.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `specs/milestones/m302/index.md`
- `specs/3466/spec.md`
- `specs/3466/plan.md`
- `specs/3466/tasks.md`

## Risks / Mitigations
- Risk: redirect contract changes break existing assertions.
  - Mitigation: RED-first test updates and explicit marker normalization.
- Risk: untrusted query values leak into HTML attributes.
  - Mitigation: whitelist normalization for status/action/reason marker values.
- Risk: action error handling hides true failures.
  - Mitigation: preserve telemetry reason codes and explicit failed marker codes.

## Interfaces / Contracts
- Form workflow contract:
  - endpoint: `POST /ops/control-action`
  - redirect markers:
    - `control_action_status` in `idle|applied|missing|failed`
    - `control_action` in `pause|resume|refresh|none`
    - `control_action_reason` in normalized reason-code values
- Ops shell HTML contract:
  - marker section id: `tau-ops-control-action-status`
  - data attributes mirror normalized query marker values.

## ADR
No ADR required for this slice (no dependency, protocol, or architecture change).

## Execution Summary
1. Added RED conformance expectations in gateway and dashboard-ui tests for:
   - missing/invalid/applied control-action redirect marker contracts,
   - ops shell control-action status panel marker rendering,
   - control-action query normalization defaults.
2. Extended ops-shell query parsing in `ops_shell_controls.rs` with normalized:
   - `control_action_status`,
   - `control_action`,
   - `control_action_reason`.
3. Updated ops-shell control-action handler in `ops_dashboard_shell.rs` to fail
   closed with deterministic redirect markers for missing/invalid submissions.
4. Added marker propagation into `TauOpsDashboardChatSnapshot` and rendered a
   command-center status panel contract in `tau-dashboard-ui`.
5. Updated gateway integration tests and dashboard-ui functional/regression
   tests to verify end-to-end marker behavior.

## Verification Notes
- RED evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway unit_requested_control_action_reason_defaults_and_normalizes_values -- --nocapture`
  - Result before alias normalization: failed (`left: \"none\"`, `right: \"missing_action\"`).
- GREEN evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway unit_requested_control_action_reason_defaults_and_normalizes_values -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` passed (`3` tests).
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` passed (`2` tests).
- Regression/gate evidence:
  - `cargo fmt --check` passed.
  - `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway -p tau-dashboard-ui --tests --no-deps -- -D warnings` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway requested_control_action -- --nocapture` passed.
