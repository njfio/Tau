# ADR 0005: Define shared operator turn/task state in tau-contract

- **Status**: Accepted
- **Date**: 2026-04-27
- **Deciders**: Gyre SE + user by continuation policy

## Context

Tau now has several operator-visible surfaces that describe the same runtime turn through different local shapes. The gateway emits OpenResponses-compatible SSE frames such as `response.output_text.delta`, `response.tool_execution.started`, and `response.tool_execution.completed`. The TUI parses those frames into local `GatewayTurnEvent` values and reconciles tool calls in app state. Webchat parses a narrower subset directly in embedded JavaScript. Mission APIs expose persisted running, completed, checkpointed, and blocked state. Dashboard streams are snapshot-oriented.

Recent work made these surfaces more reliable: gateway recovery retries now compact read-only timeout evidence, verifier recovery can force one-shot tool choice, TUI can stream assistant/tool events, and the TUI launcher fails closed when bootstrap readiness is absent. The next reliability problem is consistency: without one additive operator-state contract, TUI and webchat can render the same timeout, blocked mission, partial output, or tool failure differently.

This decision is cross-cutting because it affects gateway, TUI, webchat, dashboard, and future runtime clients. It should be additive and fixture-first so migration can happen without breaking existing clients.

## Decision

Define `OperatorTurnState` v1 and its supporting operator event/status/phase types in `tau-contract` as an additive, serde-serializable shared contract.

The contract crate is the right home because it already centralizes fixture parsing and shared contract validation without owning runtime behavior. This stage will not add external dependencies and will not move existing gateway SSE, mission, or dashboard endpoints. Existing clients keep reading legacy surfaces while new code can opt into the shared state contract.

The initial v1 contract contains:
- `OperatorTurnState`
- `OperatorTurnEvent`
- `OperatorTurnPhase`
- `OperatorTurnStatus`
- `OperatorToolState`
- `OperatorErrorContext`
- fixture parsing/validation helpers for schema examples

The contract is additive: gateway/TUI/webchat migration can follow in later stages, but v1 must already encode success, partial output, tool start/completion/failure, timeout, and blocked mission outcomes.

## Consequences

### Positive
- Gives TUI and webchat one stable vocabulary for turn/task status without forcing a UI rewrite.
- Keeps wire-format migration reversible because existing `response.*` SSE and mission/dashboard endpoints remain intact.
- Places shared schema in a crate designed for contract fixtures instead of a gateway-specific module.
- Enables focused contract tests before touching runtime or UI clients.

### Negative
- Adds more responsibility to `tau-contract`, which has so far been fixture-helper oriented.
- Client migration is a separate step; defining the contract alone does not make TUI/webchat consume it automatically.
- If v1 is too broad, it can become a parallel runtime model instead of a shared presentation contract.

### Neutral
- This decision does not add a new external dependency.
- This decision does not require Cargo manifest or lockfile changes in the definition stage.
- Existing `response.output_text.delta`, `response.tool_execution.started`, `response.tool_execution.completed`, `response.completed`, and `response.failed` events remain compatibility surfaces.

## Alternatives considered

1. **Define the schema inside tau-gateway.** Rejected because TUI and webchat would depend on gateway internals or duplicate the schema again. Gateway should map runtime events, not own the cross-client contract.
2. **Define the schema separately in each client.** Rejected because that preserves the current drift problem and prevents fixture parity tests.
3. **Introduce a new crate for operator state.** Rejected for now because `tau-contract` already exists and adding another crate/dependency surface is unnecessary for v1.
4. **Replace gateway SSE with the new contract immediately.** Rejected because it would risk breaking existing clients. The shared contract should be additive first, then migrated behind compatibility tests.

## Backwards Compatibility

The v1 contract must preserve these existing surfaces during migration:
- `response.output_text.delta`
- `response.tool_execution.started`
- `response.tool_execution.completed`
- `response.completed`
- `response.failed`
- `/gateway/missions`
- `/gateway/missions/{mission_id}`
- dashboard stream/status endpoints

Any later change that removes, renames, or reinterprets those surfaces needs a separate ADR or migration guide.

## References

- `specs/3581/spec.md`
- `specs/3581/plan.md`
- `crates/tau-contract/src/lib.rs`
- `crates/tau-tui/src/interactive/gateway_client.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/webchat_page.rs`
