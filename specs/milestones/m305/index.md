# M305 - Ops dashboard last-action visibility depth wave

Status: Active

## Context
The command-center control panel currently exposes last-action metadata through
data attributes only. Operators need readable, deterministic details in the
rendered section to quickly understand what happened without inspecting DOM
attributes.

Primary sources:
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Issue Hierarchy
- Epic: #3476
- Story: #3477
- Task: #3478

## Scope
- Render human-readable last-action detail rows in command-center UI.
- Add deterministic fallback row behavior when no action exists.
- Add dashboard-ui + gateway conformance tests for rendered markers.

## Exit Criteria
- `specs/3478/spec.md` is implemented with AC evidence.
- Last Action section includes readable detail rows and fallback content.
- New dashboard-ui/gateway tests pass and guard rendered contracts.
