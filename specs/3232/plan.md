# Plan: Issue #3232 - move gateway tool registrar api types into module

## Approach
1. RED: tighten root size guard to `1195` and add checks that tool registrar API definitions are absent from root.
2. Add `tool_registrar.rs` with tool registrar trait/public structs and implementations.
3. Re-export moved items from `gateway_openresponses.rs` and remove inlined definitions.
4. Verify with size guard, focused integration tests, and quality gates.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tool_registrar.rs` (new)
- `scripts/dev/test-gateway-openresponses-size.sh`
- `specs/milestones/m235/index.md`
- `specs/3232/spec.md`
- `specs/3232/plan.md`
- `specs/3232/tasks.md`

## Risks & Mitigations
- Risk: accidental API path breakage for root module users.
  - Mitigation: `pub use` re-exports from root and integration tests.
- Risk: visibility/import drift in moved implementations.
  - Mitigation: keep identical signatures and compile via clippy/tests.

## Interfaces / Contracts
- Root-level API names remain unchanged via re-export.
- No route/schema changes.

## ADR
No ADR required (internal module extraction + re-export only).
