# Plan #2305

Status: Reviewed
Spec: specs/2305/spec.md

## Approach

1. Add gateway session-usage persistence helper that converts pre/post `AgentCostSnapshot` values into `SessionUsageSummary` deltas and writes them via `SessionStore::record_usage_delta`.
2. In OpenResponses execution flow, capture pre-prompt and post-prompt snapshots and persist usage deltas into the same session runtime used for message persistence.
3. Add failing conformance tests first for single-request and multi-request cumulative usage persistence.
4. Keep response payload contracts unchanged and run focused gateway regression tests.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/session_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations

- Risk: Duplicate accounting if persistence is invoked multiple times per request.
  - Mitigation: Persist exactly one delta per request path from pre/post snapshots.
- Risk: Breaking OpenResponses payload expectations.
  - Mitigation: No response schema changes; rely on existing endpoint tests as regression guard.

## Interfaces / Contracts

- `persist_session_usage_delta(...)` (gateway session runtime helper) writes `SessionStore` usage deltas.
- `execute_openresponses_request(...)` keeps API response contract unchanged while adding session usage persistence side effects.
