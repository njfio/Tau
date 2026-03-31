# Plan: Issue #3652 - Retry mutating gateway turns when the model promises work without using tools

## Approach
1. Wrap the current gateway prompt execution in a bounded retry loop that is
   only enabled for action-oriented mutating prompts.
2. After each attempt, compare tool execution counts before and after the turn.
   If the attempt produced zero tool execution evidence:
   - strip assistant messages from that attempt out of agent history,
   - keep the original user request in history,
   - append a corrective user retry prompt, and
   - retry until the bounded budget is exhausted.
3. For streaming calls, buffer attempt text for action-oriented prompts and only
   flush the accepted attempt output. Discard buffered text from failed
   zero-tool attempts so operators do not see misleading promises.
4. Preserve the existing fail-closed path once the retry budget is exhausted.
5. Extend gateway tests with:
   - a retry-success case,
   - a retry-exhaustion case, and
   - reuse the conversational control case from `#3651`.

## Affected Areas
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks / Mitigations
- Risk: retry prompts could pollute final conversation state.
  Mitigation: retain original user prompts but strip failed-attempt assistant
  messages before the next retry.
- Risk: streaming clients could still see discarded attempt text.
  Mitigation: buffer action-attempt deltas locally and flush only accepted
  attempt output.
- Risk: retries could loop indefinitely.
  Mitigation: use a fixed small retry budget and preserve the gateway failure
  path on exhaustion.

## Interfaces / Contracts
- Mutating action requests may perform additional corrective retry turns inside
  `execute_openresponses_request`.
- Final successful output for retried action requests must exclude failed-attempt
  assistant text.
- Exhausted retries still return the gateway error envelope.

## Verification
- `cargo test -p tau-gateway regression_openresponses_retries_zero_tool_action_completion_until_tool_execution -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_zero_tool_action_retry_exhaustion_fails_closed -- --test-threads=1`
- `cargo test -p tau-gateway functional_openresponses_allows_zero_tool_conversational_completion -- --test-threads=1`
