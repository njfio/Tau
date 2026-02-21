# Plan: Issue #3040 - Training proxy coverage hardening

## Approach
1. Add RED tests for response `x-request-id` pass-through, transport-failure logging contract, and health endpoint contract.
2. Implement minimal runtime change to propagate `x-request-id` from upstream response headers.
3. Re-run targeted proxy tests and baseline fmt/check gates.

## Affected Paths
- `crates/tau-training-proxy/src/lib.rs`
- `specs/milestones/m188/index.md`
- `specs/3040/spec.md`
- `specs/3040/plan.md`
- `specs/3040/tasks.md`

## Risks and Mitigations
- Risk: over-scoping into broader proxy header policy.
  - Mitigation: restrict to explicit `x-request-id` pass-through only.
- Risk: flaky upstream-failure tests.
  - Mitigation: use deterministic unreachable localhost port and assert contract fields only.

## Interfaces / Contracts
- Response header contract: include `x-request-id` when upstream provides it.
- Failure contract: `training_proxy_upstream_request_failed` error code with attribution log entry containing `error_code=upstream_request_failed`.

## ADR
Not required (small behavior hardening + tests in existing crate boundary).
