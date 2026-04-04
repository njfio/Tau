# Tasks: Issue #3671 - Add raw gateway payload tracing and reconcile TUI tool lifecycle state

- [x] T1 (RED): add gateway/TUI regressions for payload-rich attempt tracing and
      reconciled tool lifecycle state.
- [x] T2 (GREEN): persist structured request/response payload evidence per
      attempt and reconcile TUI tool starts/completions by tool call id.
- [x] T3 (VERIFY): rerun the scoped gateway/TUI verification stack.

## Tier Mapping
- Functional: timeout/failure attempts remain inspectable through trace files
  and the TUI no longer reports completed tools as active
- Regression: retried gateway attempts persist structured request/response
  payload evidence and the TUI active tool count returns to zero after
  completions
