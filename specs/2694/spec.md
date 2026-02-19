# Spec: Issue #2694 - PRD gateway jobs list and cancel endpoints

Status: Implemented

## Problem Statement
`specs/tau-ops-dashboard-prd.md` API contract requires jobs operational surfaces:
- `GET /gateway/jobs`
- `POST /gateway/jobs/{id}/cancel`

`tau-gateway` currently lacks these endpoints.

## Acceptance Criteria

### AC-1 Authenticated operators can list jobs
Given a valid authenticated request,
When `GET /gateway/jobs` is called,
Then the gateway returns deterministic jobs list payload with stable status fields.

### AC-2 Authenticated operators can cancel a known job
Given a valid authenticated request with a known job id,
When `POST /gateway/jobs/{id}/cancel` is called,
Then the gateway cancels the job and returns deterministic cancelled status payload.

### AC-3 Unknown jobs return deterministic not-found error
Given a valid authenticated request with an unknown job id,
When `POST /gateway/jobs/{id}/cancel` is called,
Then the gateway returns deterministic `404` error.

### AC-4 Unauthorized jobs endpoint requests are rejected
Given missing or invalid auth,
When jobs endpoints are called,
Then the gateway returns fail-closed `401`.

### AC-5 Gateway status discovery advertises jobs endpoints
Given an authenticated status request,
When `GET /gateway/status` is called,
Then `gateway.web_ui` includes `jobs_endpoint` and `job_cancel_endpoint_template`.

### AC-6 Scoped verification gates pass
Given this implementation slice,
When scoped checks run,
Then `cargo fmt --check`, `cargo clippy -p tau-gateway -- -D warnings`, and targeted gateway tests pass.

## Scope

### In Scope
- Add gateway routes:
  - `GET /gateway/jobs`
  - `POST /gateway/jobs/{job_id}/cancel`
- Expose deterministic jobs payload backed by existing runtime external coding agent sessions.
- Add status discovery metadata for jobs endpoints.
- Add conformance/regression tests for list, cancel, not-found, and auth behavior.

### Out of Scope
- Dashboard UI implementation.
- New generic background-job execution engine.
- Multi-backend job persistence redesign.

## Conformance Cases
- C-01 (integration): jobs list endpoint returns deterministic authenticated payload.
- C-02 (integration): job cancel endpoint cancels known job and returns deterministic payload.
- C-03 (regression): unknown job id returns deterministic `404`.
- C-04 (regression): unauthorized jobs endpoint requests return `401`.
- C-05 (regression): status discovery includes jobs endpoint metadata.
- C-06 (verify): scoped fmt/clippy/targeted tests pass.

## Success Metrics / Observable Signals
- Dashboard can list and cancel jobs through dedicated authenticated endpoints.
- Jobs endpoint discovery is API-driven via `/gateway/status`.
- Endpoint contracts are deterministic and conformance-covered.
