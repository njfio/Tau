# M306 - Ops dashboard last-action reason visibility wave

Status: Active

## Context
Command-center Last Action rows now show request id/action/actor/timestamp, but
the action reason is still missing from UI snapshot/render contracts even though
backend action audit records include it.

Primary sources:
- `crates/tau-gateway/src/gateway_openresponses/dashboard_status.rs`
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Issue Hierarchy
- Epic: #3480
- Story: #3481
- Task: #3482

## Scope
- Add `last_action_reason` into command-center snapshot mapping.
- Render readable Last Action reason row in ops shell.
- Add deterministic fallback reason behavior (`none`) and contract tests.

## Exit Criteria
- `specs/3482/spec.md` status is `Implemented` with AC evidence.
- Last Action reason row is rendered with deterministic values.
- dashboard-ui and gateway tests guard reason row contracts.
