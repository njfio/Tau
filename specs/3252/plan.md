# Plan: Issue #3252 - move shell/auth entry handlers to dedicated module

## Approach
1. RED: tighten root guard to `650` and assert moved shell/auth handler definitions are not declared in root.
2. Add `entry_handlers.rs` and move webchat/dashboard/auth-bootstrap handlers.
3. Import moved handlers from root for router wiring.
4. Verify existing functional auth bootstrap contract test and run verification gates.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/entry_handlers.rs` (new)
- `scripts/dev/test-gateway-openresponses-size.sh`
- `specs/milestones/m240/index.md`
- `specs/3252/spec.md`
- `specs/3252/plan.md`
- `specs/3252/tasks.md`

## Risks & Mitigations
- Risk: route binding compile errors if moved handlers are not in root scope.
  - Mitigation: explicit imports in root and targeted functional tests.
- Risk: auth bootstrap payload drift.
  - Mitigation: add dedicated functional contract test for bootstrap endpoint.

## Interfaces / Contracts
- Endpoint paths and response contracts unchanged.
- Root module remains route wiring surface only.

## ADR
No ADR required (internal handler extraction only).
