# Tasks: Issue #3669 - Stream gateway Ralph-loop progress into the TUI

- [x] T1 (RED): add TUI regressions proving the gateway request uses
      `stream: true` and that streamed text/tool events update the UI.
- [x] T2 (GREEN): emit tool lifecycle SSE frames from the gateway and consume
      streamed turn events in the TUI client/app state.
- [x] T3 (VERIFY): rerun scoped `tau-tui` and `tau-gateway` verification.

## Tier Mapping
- Functional: streamed assistant text and tool lifecycle events appear in the
  TUI during an in-flight gateway turn
- Regression: the TUI no longer uses a blocking non-streaming `/v1/responses`
  request for interactive turns
