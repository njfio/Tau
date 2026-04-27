# Tasks: Issue #3669 - Stream gateway Ralph-loop progress into the TUI

- [x] Gateway SSE tool-frame prerequisite: gateway already emits
      `response.tool_execution.started` and `response.tool_execution.completed`
      frames from `openresponses_execution_handler.rs`.
- [x] TUI streaming client slice (RED): add TUI regressions proving the gateway
      request uses `stream: true` and that streamed text/tool events update the
      UI.
- [x] TUI streaming client slice (GREEN): consume streamed turn events in the
      TUI client/app state without changing final completion/error semantics.
- [x] TUI streaming client slice (VERIFY): rerun scoped `tau-tui` verification.

## Tier Mapping
- Functional: streamed assistant text and tool lifecycle events appear in the
  TUI during an in-flight gateway turn
- Regression: the TUI no longer uses a blocking non-streaming `/v1/responses`
  request for interactive turns
