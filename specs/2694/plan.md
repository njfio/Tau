# Plan: Issue #2694 - PRD gateway jobs list and cancel endpoints

## Approach
1. Extend runtime bridge to expose deterministic session listing helper.
2. Add `jobs_runtime` module in `gateway_openresponses` to map runtime session snapshots into jobs list/cancel response contracts.
3. Wire new routes and status discovery metadata in `gateway_openresponses`.
4. Add conformance/regression tests first (RED), implement handlers (GREEN), then run scoped gates.

## Affected Modules
- `crates/tau-runtime/src/external_coding_agent_bridge_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/jobs_runtime.rs` (new)
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks / Mitigations
- Risk: runtime listing may expose unstable ordering.
  - Mitigation: return sorted deterministic order.
- Risk: job identity/status mapping drift.
  - Mitigation: explicit schema fields and dedicated tests for known state transitions.
- Risk: auth regressions.
  - Mitigation: unauthorized regression tests for both endpoints.

## Interfaces / Contracts
- `GET /gateway/jobs`
  - `200`: `{ schema_version, generated_unix_ms, total_jobs, jobs: [...] }`
- `POST /gateway/jobs/{job_id}/cancel`
  - `200`: `{ schema_version, job_id, status, cancelled_unix_ms }`
  - `404`: deterministic not-found error for unknown id.

## ADR
- Not required. No dependency/protocol/architecture decision change.
