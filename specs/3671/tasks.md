# Tasks: Issue #3671 - Add raw gateway payload tracing and reconcile TUI tool lifecycle state

- [x] Prior TUI streaming slice: active-count symptom is partially fixed by
      completion matching on tool name.
- [x] R3671 corrected implementation (RED): add regressions for same-name
      streamed tool executions and persisted attempt payload evidence.
- [x] R3671 corrected implementation (GREEN): tool_call_id reconciliation remains open in the current TUI reducer; add it and persist gateway payload evidence.
- [x] R3671 corrected implementation (VERIFY): request_payload and response_payload fields are not yet persisted on mission iterations; verify they are present along with scoped TUI/gateway gates.

## Tier Mapping
- Functional: timeout/failure attempts remain inspectable through trace files
  and the TUI no longer reports completed tools as active
- Regression: retried gateway attempts persist structured request/response
  payload evidence and the TUI active tool count returns to zero after
  completions
