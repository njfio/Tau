# 3598 Gateway OpenResponses Token Budget

## Objective

Fix the `/v1/responses` gateway execution path so preflight token budgeting distinguishes between:
- the input guardrail derived from `max_input_chars`, and
- the total request budget enforced by the agent.

The gateway must not reject ordinary requests with `token budget exceeded` solely because total estimated tokens exceed a bogus 8k ceiling derived from the input character cap.

## Inputs/Outputs

Inputs:
- `GatewayOpenResponsesServerState.config.max_input_chars`
- resolved model metadata already present in gateway config (`model`, cost metadata)
- translated OpenResponses prompt/session state

Outputs:
- `AgentConfig` passed into the OpenResponses execution agent with correct preflight fields
- HTTP/SSE success for requests that fit within the real gateway preflight intent
- hard-fail `gateway_runtime_error` for genuinely oversized requests

## Boundaries/Non-goals

In scope:
- OpenResponses gateway preflight config and tests
- the real `/v1/responses` request path used by the TUI gateway transport

Out of scope:
- local runtime `tau-coding-agent` startup token budgeting
- broad session compaction redesign
- hidden fallback behavior or silent request truncation
- model-catalog redesign

## Failure modes

- Request exceeds the gateway input preflight guardrail and must fail before provider dispatch.
- Request creation/prompt execution fails for unrelated provider/runtime reasons and should keep current hard-fail behavior.
- Session state persists large history; the gateway must still fail only on real enforced budgets, not on the old bogus total-token cap.

## Acceptance criteria

- [ ] `/v1/responses` no longer sets `AgentConfig.max_estimated_total_tokens` to the derived input-preflight token cap.
- [ ] A request with estimated total tokens above the old 8k ceiling but below the intended preflight policy succeeds in the OpenResponses handler test path.
- [ ] A truly oversized request still returns `gateway_runtime_error` containing `token budget exceeded` before provider dispatch.
- [ ] Regression coverage proves the handler does not reuse the same derived preflight value for both input and total token ceilings.

## Files to touch

- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3598-gateway-openresponses-token-budget.md`

## Error semantics

- Keep hard-fail behavior.
- Oversized requests must continue to return `OpenResponsesApiError::gateway_runtime_error` with a visible `token budget exceeded` message.
- Do not silently compact, truncate, or retry around the budget error in this issue.

## Test plan

1. Add a failing regression test proving a request that previously tripped the bogus total-token cap now succeeds when only the old total ceiling was violated.
2. Keep/add a failing test proving a genuinely oversized request still fails before provider dispatch.
3. Implement the minimal handler change.
4. Run targeted `tau-gateway` tests for the OpenResponses preflight path.
5. Run the relevant integration tests exercising the real `/v1/responses` HTTP route.
