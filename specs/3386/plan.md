# Plan: Issue #3386

## Approach
1. Build a scenario-to-test coverage matrix from `specs/tau-e2e-testing-prd.md` and identify uncovered cases relative to existing tests.
2. Add shared deterministic E2E harness/helpers in integration tests (scripted LLM responses, isolated state dirs, ephemeral gateway port orchestration, auth-aware HTTP/WS helpers).
3. Implement missing P0 conformance tests first (RED -> GREEN), then P1/P2/P3 gaps in priority order.
4. Fix runtime/test defects uncovered by new scenarios with minimal behavioral changes and regression protection.
5. Add tier selectors and validate targeted PR/nightly/weekly command sets.

## Affected Modules
- `specs/milestones/m291/index.md`
- `specs/3386/spec.md`
- `specs/3386/plan.md`
- `specs/3386/tasks.md`
- `tests/integration/Cargo.toml`
- `tests/integration/tests/*` (new E2E harness/scenario files)
- `crates/tau-gateway/src/gateway_openresponses/tests.rs` (only if internal endpoint/runtime fixtures need extension)
- Additional runtime modules only if test-driven gaps expose defects.

## Risks and Mitigations
- Risk: broad scenario scope causes flaky/slow test runs.
  Mitigation: deterministic scripted provider, isolated state roots, tiered selectors, and bounded timeouts.
- Risk: PRD scenarios reference capabilities outside current runtime contract.
  Mitigation: explicit `N/A` entries with concrete code-path justification and issue follow-up where required.
- Risk: introducing fragile cross-module coupling in test harness.
  Mitigation: keep helpers thin, prefer public contracts, and avoid white-box coupling where possible.

## Interfaces / Contracts
- Gateway HTTP/WS contracts under `tau-gateway` OpenResponses endpoints.
- Session/memory/safety/training/cortex/dashboard operator contracts currently exposed by runtime.
- Integration harness API for PRD scenario tests (setup, auth, scripted responses, capture assertions).

## ADR
Not required for current scope unless adding new dependencies or changing wire contracts.
