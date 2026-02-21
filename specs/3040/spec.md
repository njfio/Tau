# Spec: Issue #3040 - Training proxy failure-path and metadata coverage hardening

Status: Implemented

## Problem Statement
`tau-training-proxy` lacks explicit conformance checks for selected operational behavior, especially upstream response metadata propagation and structured failure logging guarantees. This increases regression risk in prompt-optimization proxy operations.

## Acceptance Criteria

### AC-1 Upstream request-id metadata is preserved in proxy responses
Given an upstream `/v1/chat/completions` response containing `x-request-id`,
When the proxy forwards the response,
Then the client response includes the same `x-request-id` header.

### AC-2 Upstream transport failures produce deterministic error contracts and attribution logs
Given a proxy request with valid attribution headers and an unreachable upstream,
When forwarding fails,
Then the proxy responds `502` with `training_proxy_upstream_request_failed` and appends an attribution record with `error_code=upstream_request_failed`.

### AC-3 Health contract remains stable
Given proxy health requests,
When calling `/training/proxy/health`,
Then status payload includes schema version, ready status, upstream URL, and attribution log path.

### AC-4 Verification gates remain green
Given the test/runtime changes,
When running validation,
Then targeted crate tests plus fmt/check pass.

## Scope

### In Scope
- `crates/tau-training-proxy/src/lib.rs`
- `specs/milestones/m188/index.md`
- `specs/3040/*`

### Out of Scope
- New proxy endpoints.
- Broader training-runner runtime behavior outside proxy crate.

## Conformance Cases
- C-01: response header pass-through for `x-request-id`.
- C-02: upstream request failure returns deterministic 502 error payload and attribution log entry.
- C-03: health endpoint returns expected schema/fields.
- C-04: validation commands pass.

## Success Metrics / Observable Signals
- `cargo test -p tau-training-proxy integration_proxy_forwards_request_and_persists_attribution_log -- --nocapture`
- `cargo test -p tau-training-proxy integration_proxy_returns_upstream_request_id_header -- --nocapture`
- `cargo test -p tau-training-proxy regression_proxy_returns_bad_gateway_and_logs_upstream_transport_failure -- --nocapture`
- `cargo test -p tau-training-proxy integration_proxy_health_endpoint_reports_ready_contract -- --nocapture`
- `cargo fmt --check`
- `cargo check -q`

## Approval Gate
P1 scope: spec authored/reviewed by agent; implementation proceeds and is flagged for human review in PR.
