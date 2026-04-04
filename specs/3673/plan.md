# Plan: Force Tool-Required Retry Turns In Ralph-Loop Recovery

## Approach

1. Add an internal tool-choice override field/setter to `Agent`.
2. Update request construction in `tau-agent-core` to prefer the override when tools are present.
3. In the gateway Ralph-loop retry path, set `ToolChoice::Required` for retries that still need tool or mutation evidence.
4. Update CLI provider prompt renderers so `ToolChoice::Required` produces a stricter response contract.
5. Extend gateway/provider regressions to assert the new behavior.

## Affected Modules

- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-provider/src/codex_cli_client.rs`
- `crates/tau-provider/src/claude_cli_client.rs`
- `crates/tau-provider/src/gemini_cli_client.rs`

## Risks

- Forcing required tool choice too broadly could block valid plain-text completions.

## Mitigation

- Apply the override only on gateway retry turns, not on first attempts or non-action flows.
- Keep normal auto behavior unchanged elsewhere.
