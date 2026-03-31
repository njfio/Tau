# Spec: Issue #3659 - Expose gateway mission inspection and TUI resume controls for checkpointed Ralph-loop missions

Status: Reviewed

## Problem Statement
Tau's Ralph-loop gateway now persists rich mission state with verifier bundles,
learning context, and explicit completion/checkpoint markers, but operators
still cannot use that state from the main gateway/TUI flow. Checkpointed or
blocked missions are just JSON files on disk. Tau needs a first-class operator
surface to inspect persisted missions and resume them from the TUI without
creating an unrelated implicit mission.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/endpoints.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_bootstrap.rs`
- `crates/tau-gateway/src/gateway_openresponses/status_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/status.rs`
- `crates/tau-tui/src/interactive/ui_status.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
- `specs/3659/spec.md`
- `specs/3659/plan.md`
- `specs/3659/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- automatic background mission re-entry without operator input
- web dashboard mission management UI beyond exposing gateway endpoints
- changing the public OpenResponses request/response schema
- cross-crate checkpoint orchestration in `tau-session` or `tau-orchestrator`

## Acceptance Criteria
### AC-1 Gateway exposes persisted mission inventory and detail endpoints
Given persisted Ralph-loop mission state in the gateway state directory,
when an authorized operator requests `/gateway/missions` or
`/gateway/missions/{mission_id}`,
then the gateway returns mission summaries or full mission detail including
mission id, session key, status, goal summary, latest verifier, latest
completion, and iteration metadata.

### AC-2 Interactive TUI can inspect persisted missions through gateway controls
Given a gateway-backed interactive TUI session,
when the operator runs mission control commands,
then the TUI can list recent missions and display detail for a specific mission
without requiring direct filesystem access.

### AC-3 TUI resume control binds the active mission and session for follow-up turns
Given a persisted checkpointed or blocked mission,
when the operator resumes it from the interactive TUI,
then the TUI records that mission as the active mission, switches to the
mission's linked session key, and surfaces the active mission to the operator.

### AC-4 Resumed turns continue the same Ralph-loop mission identity
Given the operator has resumed a mission in the interactive TUI,
when they submit the next prompt,
then the gateway request metadata includes the explicit `mission_id` together
with the linked `session_id` so the outer loop continues the persisted mission
instead of creating a fresh implicit mission.

## Conformance Cases
- C-01 / AC-1 / Regression:
  seed persisted mission files, call `/gateway/missions`, and verify the
  authorized response lists checkpointed and blocked missions with summaries.
- C-02 / AC-1 / Regression:
  call `/gateway/missions/{mission_id}` for a persisted mission and verify the
  detail response includes verifier and completion data from mission state.
- C-03 / AC-2 / Functional:
  use the interactive TUI mission list command against a gateway fixture and
  verify the mission summaries are surfaced in the chat transcript.
- C-04 / AC-3 / Functional:
  use the interactive TUI resume command and verify the app switches to the
  mission's persisted `mission_id` and `session_key` and shows the active
  mission in the status surface.
- C-05 / AC-4 / Regression:
  after resuming a mission in the interactive TUI, submit a prompt and verify
  the gateway request body includes both `metadata.mission_id` and
  `metadata.session_id` for the resumed mission.

## Success Metrics / Observable Signals
- Operators can discover checkpointed or blocked missions through the gateway
  without opening mission JSON files directly
- The interactive TUI can intentionally resume a persisted mission instead of
  relying on implicit session-only continuation
- Resumed turns keep the same mission identity, which lets the Ralph loop,
  verifier history, and learning state continue coherently

## Files To Touch
- `specs/3659/spec.md`
- `specs/3659/plan.md`
- `specs/3659/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/endpoints.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/server_bootstrap.rs`
- `crates/tau-gateway/src/gateway_openresponses/status_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/status.rs`
- `crates/tau-tui/src/interactive/ui_status.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
