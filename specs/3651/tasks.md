# Tasks: Issue #3651 - Fail interactive gateway action turns that produce no tool evidence

- [ ] T1 Red: add gateway regressions for a zero-tool mutating action request
      and a zero-tool conversational control request.
- [ ] T2 Green: track tool execution evidence in
      `openresponses_execution_handler.rs` and fail closed on zero-tool
      mutating completions.
- [ ] T3 Verify: run the targeted gateway tests and confirm the action request
      now returns a gateway error while conversational replies still succeed.
