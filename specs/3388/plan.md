# Plan: Issue #3388

## Approach
1. Add RED tests for `O3-06`, `O3-07`, `O3-08`, and `O3-10` in the existing PR-tier OpenAI compatibility matrix test.
2. Implement OpenAI adapter request validation for unsupported tool-call request fields and multi-choice requests.
3. Thread `max_tokens` through translated requests into runtime execution and propagate provider `finish_reason` into OpenAI-compatible payloads.
4. Assert tool-role context forwarding using the capture/scripted test clients.
5. Update conformance mapping for the four O3 scenarios and re-run tiered verification.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/openai_compat.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/openai_compat_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/milestones/m291/index.md`
- `specs/3388/spec.md`
- `specs/3388/plan.md`
- `specs/3388/tasks.md`

## Risks and Mitigations
- Risk: behavior changes could regress existing OpenAI compatibility tests.
  Mitigation: keep deltas narrow, preserve existing O3 pass paths, and run focused + crate-wide tests.
- Risk: `max_tokens` wiring could be silently ignored by runtime.
  Mitigation: assert forwarded provider request fields directly with capture/scripted clients.
- Risk: PRD wording for O3-06 assumes full tool-call passthrough.
  Mitigation: codify explicit fail-closed contract for unsupported request-side tool-call wiring to eliminate silent ambiguity.

## Interfaces / Contracts
- OpenAI-compatible endpoint contract: `/v1/chat/completions`.
- Gateway response error envelope (`OpenResponsesApiError`) for deterministic client errors.
- Agent runtime request contract (`ChatRequest.max_tokens`, message context forwarding).

## ADR
Not required (no new dependency, no cross-service protocol introduction).
