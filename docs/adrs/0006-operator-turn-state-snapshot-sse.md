# ADR 0006: Additive OperatorTurnState Snapshot SSE Event

- **Status**: Accepted
- **Date**: 2026-04-27
- **Deciders**: GitHub Copilot (Gyre SE), user-approved continuation

## Context

Issue #3581 established `OperatorTurnState` v1 in `tau-contract` so TUI and webchat can share a durable operator turn/task state vocabulary. Issue #3582 then added a tau-tui adapter that can apply an `OperatorTurnState` snapshot into transcript, status, mission, and tool-panel state. The remaining gap is live transport: tau-tui still receives gateway SSE frames as gateway-specific `GatewayTurnEvent` values, and no live frame carries the shared state snapshot.

The existing gateway SSE stream is already consumed by TUI and webchat. Removing or renaming the current `response.*` frames would be a breaking API change. The safe route is to add a new optional frame that clients can ignore until they opt in. The event name must be stable enough for clients and tests to depend on.

## Decision

Add an optional SSE event named `response.operator_turn_state.snapshot` whose `data` payload is a serialized `OperatorTurnState` snapshot.

The event is additive. Producers may emit it alongside the current `response.output_text.delta`, `response.tool_execution.started`, `response.tool_execution.completed`, `response.completed`, and `response.failed` frames. Consumers must keep accepting legacy frames and should treat the snapshot as a richer state projection rather than a replacement for all existing frames.

For the first implementation slice, tau-tui will parse `response.operator_turn_state.snapshot` when it appears in the gateway stream and apply it through `interactive::operator_state::apply_operator_turn_state`. Gateway emission can be added in a later slice using the same event name.

## Consequences

### Positive
- TUI can consume the shared `OperatorTurnState` contract from live streams without waiting for a broad gateway/webchat rewrite.
- Legacy clients remain compatible because unknown SSE event names are ignorable.
- Snapshot tests can exercise transcript, status, and tool state through the same adapter used by future gateway emissions.

### Negative
- During transition, clients may receive both legacy deltas and full snapshots, so duplicate assistant text and duplicate tool rows need explicit tests.
- The event name becomes part of the public gateway stream contract once emitted by the gateway.
- `OperatorToolStatus` currently has no timed-out variant, so tool-level timeouts may need turn-level error context or a future contract extension.

### Neutral
- The gateway stream may contain a mix of event-level deltas and state snapshots for a while.
- Webchat can ignore the new frame until its own shared-state consumer is implemented.

## Alternatives considered

1. **Replace existing response.* frames with OperatorTurnState snapshots**: Rejected because it would break existing TUI/webchat consumers and make rollback harder.
2. **Use a non-response event namespace such as operator.turn_state.snapshot**: Rejected for now because the gateway stream is already organized around `response.*` events and colocating the snapshot with response events keeps client routing simple.
3. **Wait for gateway emission before adding TUI parsing**: Rejected because TUI can safely prove the consumption path first with mocked SSE frames, reducing risk before gateway runtime changes.

## References

- docs/adrs/0005-shared-operator-turn-task-state.md
- docs/architecture/shared-operator-state-v1.md
- docs/architecture/tui-operator-state-consumption-v1.md
- specs/3582/spec.md
