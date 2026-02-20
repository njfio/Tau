# Spec: Issue #2734 - G18 stretch routines cron management webchat panel

Status: Accepted

## Problem Statement
Tau gateway already exposes event-scheduler diagnostics and jobs management endpoints, but operators cannot manage routines from the webchat UI. G18 stretch parity still requires a cron-management surface directly in webchat.

## Acceptance Criteria

### AC-1 Webchat exposes a routines management view
Given the gateway webchat page,
When tabs/views render,
Then a dedicated routines tab provides scheduler status and jobs controls.

### AC-2 Scheduler diagnostics are visible in routines view
Given authenticated access to gateway status,
When routines data refreshes,
Then scheduler health, rollout gate, reason code, counters, and diagnostics from `payload.events` are rendered deterministically.

### AC-3 Jobs list and cancel controls are wired in routines view
Given authenticated access,
When routines jobs refreshes,
Then webchat fetches `/gateway/jobs`, renders active jobs, and can cancel a job through `/gateway/jobs/{job_id}/cancel`.

### AC-4 Routines failure states produce deterministic diagnostics
Given unauthorized/error responses,
When routines status/jobs requests fail,
Then routines status panes show deterministic failure messaging and telemetry reason codes.

### AC-5 Existing webchat surfaces remain compatible
Given conversation/dashboard/tools/cortex/sessions/memory/configuration flows,
When routines panel is added,
Then existing webchat tests and endpoint regressions remain green.

### AC-6 Scoped verification gates pass
Given this slice,
When scoped checks run,
Then `cargo fmt --check`, `cargo clippy -p tau-gateway -- -D warnings`, and targeted gateway routines/webchat tests pass.

## Scope

### In Scope
- Webchat routines tab markup and JS behavior.
- Status rendering from existing `gateway.status.events` payload.
- Jobs list/cancel wiring using existing endpoints.
- Tests for routines panel markers and regressions.
- Update `tasks/spacebot-comparison.md` G18 stretch checklist evidence.

### Out of Scope
- New scheduler backend endpoints.
- New dependency additions.
- CI/CD pipeline changes.

## Conformance Cases
- C-01 (unit): webchat contains routines tab/view + routines status/jobs DOM markers.
- C-02 (unit): routines script includes status render and jobs/cancel request handlers with existing endpoints.
- C-03 (functional): routines jobs request path renders active jobs and cancel controls.
- C-04 (regression): existing webchat and jobs/status endpoint tests remain green.
- C-05 (verify): fmt/clippy/targeted tests pass.

## Success Metrics / Observable Signals
- Operators can manage routine operations from `/webchat` without external curl commands.
- G18 stretch `Cron management` checklist item is closed with linked evidence.
- No regressions in existing webchat flows.
