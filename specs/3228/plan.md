# Plan: Issue #3228 - move gateway auth runtime-state/status types into module

## Approach
1. RED: tighten root size guard to `1230` and add checks that auth types are absent from root.
2. Move auth runtime-state/status type definitions from `gateway_openresponses.rs` into `auth_runtime.rs`.
3. Keep visibility aligned for root state wiring and tests.
4. Verify with targeted auth/status tests and quality gates.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/auth_runtime.rs`
- `scripts/dev/test-gateway-openresponses-size.sh`
- `specs/milestones/m234/index.md`
- `specs/3228/spec.md`
- `specs/3228/plan.md`
- `specs/3228/tasks.md`

## Risks & Mitigations
- Risk: visibility mismatches for moved structs/fields.
  - Mitigation: use `pub(super)` visibility for cross-module access only.
- Risk: accidental auth behavior drift.
  - Mitigation: run focused auth/session/rate-limit/status test set.

## Interfaces / Contracts
- No API contract changes.
- `/gateway/status` behavior remains stable.

## ADR
No ADR required (internal module extraction only).
