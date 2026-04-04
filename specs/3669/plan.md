# Plan: Issue #3669 - Stream gateway Ralph-loop progress into the TUI

## Approach
1. Add red tests on the TUI side proving the gateway client must request
   `stream: true` and can assemble streamed text/tool events into visible UI
   state.
2. Extend the gateway SSE surface to emit tool lifecycle frames alongside the
   existing text delta frames.
3. Replace the blocking gateway client path with an SSE reader that converts
   streamed frames into internal turn events for the TUI.
4. Update the TUI app state to consume incremental turn events, append streamed
   assistant text, update tool panel entries, and preserve final completion and
   error behavior.

## Proposed Design
### Gateway SSE tool frames
- Subscribe to agent tool lifecycle events already observed in
  `openresponses_execution_handler.rs`.
- When streaming mode is active, emit additional SSE frames for:
  - tool execution start
  - tool execution end
- Keep the existing `response.output_text.delta`, `response.output_text.done`,
  `response.completed`, and `response.failed` events intact.

### TUI gateway stream client
- Change `spawn_gateway_turn` to start a worker that posts a streaming request to
  `/v1/responses`.
- Parse SSE frames line-by-line from the blocking `reqwest` response body.
- Convert frames into internal turn events such as text delta, tool start, tool
  end, completed, and failed.

### TUI app state
- Drain turn events on every tick rather than waiting for a single terminal
  result.
- Add a streaming assistant message buffer/index so text deltas update the chat
  panel incrementally.
- Map tool start/end events into the existing `ToolPanel`.
- Keep mission control and final error handling compatible with the current
  status bar semantics.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3669"
  change_surface:
    - symbol: "interactive gateway turn execution"
      location: "crates/tau-tui/src/interactive/gateway_client.rs"
      change_type: "modification"
      current: "blocking non-streaming POST returning final JSON body"
      proposed: "streaming SSE POST returning incremental turn events"
      compatibility: "safe"
      reason: "interactive gateway client is internal to tau-tui and should expose live Ralph-loop progress"
    - symbol: "gateway /v1/responses SSE frames"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "streams text deltas only"
      proposed: "streams text deltas plus tool lifecycle frames"
      compatibility: "safe"
      reason: "additive SSE events preserve existing consumers while enabling the TUI tools panel"
  overall_compatibility: "safe"
  approach:
    strategy: "make the TUI consume gateway SSE and expose tool lifecycle progress"
    steps:
      - "add TUI red tests for streamed text/tool updates"
      - "emit gateway SSE tool lifecycle frames"
      - "parse SSE into TUI turn events and update app state incrementally"
      - "rerun gateway and tau-tui scoped verification"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: partial SSE parsing bugs leave the TUI stuck in `THINKING`.
  Mitigation: add deterministic scripted gateway tests covering completion and
  failure frames.
- Risk: tool lifecycle frames break existing SSE consumers.
  Mitigation: make the new frames additive and preserve existing event names.
- Risk: chat rendering becomes noisy with repeated empty deltas.
  Mitigation: ignore empty deltas and only create/update assistant messages when
  real text arrives.

## Verification
- `cargo test -p tau-tui app_gateway_tests -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_stream -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-tui/src/interactive/gateway_client.rs crates/tau-tui/src/interactive/app.rs crates/tau-tui/src/interactive/chat.rs crates/tau-tui/src/interactive/app_gateway_tests.rs crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs crates/tau-gateway/src/gateway_openresponses/tests.rs`
