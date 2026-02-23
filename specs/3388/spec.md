# Spec: Issue #3388 - Close remaining P0 OpenAI compatibility gaps

Status: Accepted

## Problem Statement
Issue `#3386` delivered broad Tau E2E PRD coverage, but Scenario Group 3 still left P0 gaps (`O3-06`, `O3-07`, `O3-08`, `O3-10`) as `N/A` because behavior was either implicit, silently ignored, or not asserted with deterministic tests.

## Scope
In scope:
- Define explicit, testable behavior for unresolved P0 OpenAI chat compatibility scenarios.
- Add deterministic PR-tier tests in `tau-gateway` for those scenarios.
- Implement minimal runtime/adapter changes required for tests to pass.
- Update conformance mapping for affected O3 scenarios.

Out of scope:
- Non-P0 scenario groups.
- Full OpenAI tool-call passthrough parity beyond current gateway runtime contract.

## Acceptance Criteria
### AC-1 O3-06 tools payloads fail closed with explicit client error
Given a chat-completions request with OpenAI tool-call request fields (`tools` / `tool_choice`),
when the request is submitted to `/v1/chat/completions`,
then the gateway responds with a deterministic 4xx error instead of silently ignoring unsupported tool-call request wiring.

### AC-2 O3-07 tool-result messages are accepted and forwarded to runtime context
Given a chat-completions continuation payload that includes a `tool` role message with textual result content,
when the request is translated and executed,
then the provider request includes the tool-result context (not dropped), and the request completes successfully.

### AC-3 O3-08 multi-choice requests (`n > 1`) fail gracefully
Given a chat-completions request with `n > 1`,
when submitted,
then the gateway returns a deterministic client error describing unsupported multi-choice mode (graceful fail path).

### AC-4 O3-10 max token intent is preserved and surfaced in completion reason
Given a chat-completions request with `max_tokens`,
when executed,
then the provider request receives that max token budget and the OpenAI-compatible response reflects the model completion reason (including `"length"` when returned by the provider).

### AC-5 Conformance traceability for O3-06/O3-07/O3-08/O3-10 is executable
Given the conformance artifacts for this issue,
when reviewed,
then all four scenarios map to concrete tests with no `N/A` for these rows.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | tools/tool_choice present in chat payload | POST `/v1/chat/completions` | 4xx with explicit unsupported-tool-call code |
| C-02 | AC-2 | Functional | tool-role message provided | POST `/v1/chat/completions` | provider receives tool context; response 200 |
| C-03 | AC-3 | Functional | `n = 2` | POST `/v1/chat/completions` | 4xx graceful unsupported multi-choice error |
| C-04 | AC-4 | Functional | `max_tokens = 10` and scripted provider `finish_reason = length` | POST `/v1/chat/completions` | provider request carries `max_tokens=10`; response finish_reason is `length` |
| C-05 | AC-5 | Conformance | O3 rows in PRD matrix | update conformance mapping | O3-06/07/08/10 mapped to tests |

## Success Metrics / Observable Signals
- New/updated PR-tier OpenAI compatibility tests pass deterministically in `cargo test -p tau-gateway tier_pr_o3_openai_compatibility_matrix`.
- No remaining `N/A` entries for `O3-06`, `O3-07`, `O3-08`, `O3-10` in issue conformance mapping.
