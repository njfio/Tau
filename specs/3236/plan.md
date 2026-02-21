# Plan: Issue #3236 - move gateway endpoint/path constants into module

## Approach
1. RED: tighten root size guard to `1110` and add ownership checks that selected endpoint constants are absent from root.
2. Add `endpoints.rs` and move endpoint/path constants there.
3. Wire root module to use moved constants without changing behavior.
4. Verify with guard, targeted integration tests, and quality gates.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/endpoints.rs` (new)
- `scripts/dev/test-gateway-openresponses-size.sh`
- `specs/milestones/m236/index.md`
- `specs/3236/spec.md`
- `specs/3236/plan.md`
- `specs/3236/tasks.md`

## Risks & Mitigations
- Risk: constant visibility breaks sibling modules.
  - Mitigation: define constants `pub(super)` and import via root module namespace.
- Risk: accidental endpoint-string drift.
  - Mitigation: keep literal values identical and run endpoint-focused integration tests.

## Interfaces / Contracts
- Endpoint strings and route mappings unchanged.
- `/gateway/status` endpoint fields unchanged.

## ADR
No ADR required (internal constants extraction only).
