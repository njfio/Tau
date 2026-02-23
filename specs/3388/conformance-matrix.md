# Conformance Matrix: Issue #3388

| Scenario | Priority | Status | Test(s) | Notes |
|---|---|---|---|---|
| O3-06 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | `tools`/`tool_choice` request fields fail closed with `unsupported_tools` client error. |
| O3-07 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | Tool-role continuation message content is forwarded into provider request context. |
| O3-08 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | Multi-choice requests (`n > 1`) fail gracefully with deterministic `unsupported_n` error. |
| O3-10 | P0 | Covered | tier_pr_o3_openai_compatibility_matrix | `max_tokens` is propagated to provider request and response finish reason reflects provider reason (including `length`). |
