# Plan: Issue #2980 - OpenAI compatibility runtime extraction

## Approach
1. Capture RED baseline for hotspot size and scoped OpenAI compatibility tests.
2. Create `openai_compat_runtime.rs` and move OpenAI handler implementations.
3. Wire module imports and route handler references from `gateway_openresponses.rs`.
4. Run targeted OpenAI compatibility suites and quality gates.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/openai_compat_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs` (only if imports/visibility need adjustment)

## Risks and Mitigations
- Risk: behavior drift in telemetry reason codes/counters.
  - Mitigation: pure move with no semantic edits; run targeted OpenAI compatibility tests.
- Risk: route wiring regressions.
  - Mitigation: conformance checks for route handlers + scoped integration tests.

## Interfaces / Contracts
- No endpoint or payload contract changes.
- Existing route constants and path bindings remain unchanged.
