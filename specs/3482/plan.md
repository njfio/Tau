# Plan: Issue #3482 - M306 command-center last-action reason visibility

Status: Implemented

## Approach
1. Add RED tests in dashboard-ui and gateway suites for missing reason row
   markers/content.
2. Extend `TauOpsDashboardCommandCenterSnapshot` with `last_action_reason`
   including default fallback.
3. Update gateway snapshot mapping in `dashboard_status.rs` to populate reason
   from `last_action.reason`, fallback `none`.
4. Update ops shell render to include:
   - `data-last-action-reason`
   - `tau-ops-last-action-reason` readable row.
5. Run scoped tests + clippy/fmt and update spec evidence.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/dashboard_status.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/milestones/m306/index.md`
- `specs/3482/spec.md`
- `specs/3482/plan.md`
- `specs/3482/tasks.md`

## Risks / Mitigations
- Risk: snapshot struct expansion breaks existing initializers.
  - Mitigation: compile-driven updates + default fallback field.
- Risk: marker drift across UI/gateway assertions.
  - Mitigation: explicit selector assertions in both suites.
- Risk: empty reason values leak to UI.
  - Mitigation: deterministic fallback `none` in mapping and tests.

## Interfaces / Contracts
- Command-center snapshot:
  - new field: `last_action_reason: String`.
- Last Action render:
  - new data attribute: `data-last-action-reason`.
  - new row id: `tau-ops-last-action-reason`.

## ADR
No ADR required (additive UI/snapshot contract update).

## Execution Summary
1. Added RED tests in dashboard-ui and gateway suites for missing reason marker
   contracts in Last Action rendering.
2. Extended `TauOpsDashboardCommandCenterSnapshot` with `last_action_reason`
   and default fallback `none`.
3. Updated gateway snapshot mapping to populate reason from
   `last_action.reason` with deterministic fallback normalization.
4. Updated ops shell rendering with additive reason contracts:
   - `data-last-action-reason`
   - `tau-ops-last-action-reason` row.
5. Added gateway dashboard-status functional test for snapshot reason mapping.
6. Updated dashboard ops runbook to include Last Action reason row.

## Verification Notes
- RED evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3482 -- --nocapture` failed (`data-last-action-reason` missing).
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3482 -- --nocapture` failed (`data-last-action-reason=\"maintenance\"` missing).
- GREEN evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3482 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3482 -- --nocapture` passed.
- Regression/gate evidence:
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3478 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3478 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-dashboard-ui 3466 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo test -p tau-gateway 3466 -- --nocapture` passed.
  - `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-dashboard-ui -p tau-gateway --tests --no-deps -- -D warnings` passed.
  - `cargo fmt --check` passed.
