# Plan: Issue #3426 - Auth lifecycle verification and hardening

## Approach
1. Add RED conformance/regression tests for uncovered gateway auth lifecycle paths.
2. Implement minimal code hardening to satisfy failing tests (no protocol expansion).
3. Re-run focused auth suites and selected regression selectors.
4. Capture verification evidence and keep artifact mapping aligned with AC/C-cases.

## Implementation Targets
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/auth_runtime.rs` (if parser hardening needed)
- `specs/milestones/m296/index.md`
- `specs/3426/spec.md`
- `specs/3426/plan.md`
- `specs/3426/tasks.md`

## Risks and Mitigations
- Risk: broad test file churn in `gateway_openresponses/tests.rs`.
  - Mitigation: add tightly scoped tests near existing auth coverage and keep logic localized.
- Risk: auth parser hardening accidentally weakens validation.
  - Mitigation: pair positive-case tests with negative-case unauthorized regressions.
- Risk: flaky expiry assertions.
  - Mitigation: keep TTL windows deterministic with explicit sleep margin and local state.

## Interfaces / Contracts
- Existing gateway endpoints only:
  - `GET /gateway/auth/bootstrap`
  - `POST /gateway/auth/session`
  - `GET /gateway/status` (auth section)
- Existing unauthorized/error envelope contract (`OpenResponsesApiError`).

## ADR
Not required; no dependency or architecture change is planned.
