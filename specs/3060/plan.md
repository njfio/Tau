# Plan: Issue #3060 - ops memory delete confirmation contracts

## Approach
1. Add RED UI tests for deterministic delete form + confirmation markers.
2. Add RED gateway integration tests for confirmed delete success and
   unconfirmed/missing-target safety behavior.
3. Implement minimal `/ops/memory` delete flow (form operation branch, confirm
   gate, delete call, redirect status markers).
4. Run regression and verification gates for existing memory slices.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: delete branch could bypass confirmation and remove entries unexpectedly.
  - Mitigation: explicit confirmation field + RED tests for non-confirmed flow.
- Risk: new status fields could break existing create/edit markers.
  - Mitigation: keep existing create/edit status fields intact and rerun spec
    regression slices.
- Risk: delete status could persist in unrelated routes.
  - Mitigation: constrain marker rendering to memory panel contracts and verify
    route-specific assertions.

## Interface / Contract Notes
- `/ops/memory` retains a single form-post endpoint; operation routing remains
  form-driven (`create`, `edit`, `delete`).
- Redirect query controls extend with delete status markers while preserving
  existing `create_status`/`created_memory_id` behavior.
