# Plan: Issue #3651 - Fail interactive gateway action turns that produce no tool evidence

## Approach
1. Reuse the existing gateway action-token classifier in
   `openresponses_execution_handler.rs` to decide whether the translated prompt
   is action-oriented enough to require tool evidence.
2. Subscribe to the agent event stream and count tool execution start/end events
   for the turn alongside the existing usage and finish-reason capture.
3. After `prompt_with_stream` returns, fail closed with a gateway error when:
   - the translated prompt is action-oriented, and
   - the turn completed without any tool execution evidence.
4. Leave non-action prompts untouched so normal conversational replies can still
   complete without forced tool usage.
5. Add regression coverage in `tests.rs` using the existing scripted LLM client
   and fixture tool registrars:
   - one failing zero-tool action case,
   - one non-action control case, and
   - rely on the existing tool-pipeline test as the positive action path.

## Affected Areas
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks / Mitigations
- Risk: the action classifier may be too broad and reject benign prompts.
  Mitigation: reuse the current narrow action-token list already used for auto
  skill selection and add a non-action control regression.
- Risk: counting only tool-end events could miss failed attempts or early
  aborts.
  Mitigation: count tool execution start events as evidence that the runtime
  actually attempted action work.
- Risk: the gateway could persist misleading assistant text before enforcement.
  Mitigation: perform the zero-tool check immediately after the prompt returns
  and before response assembly.

## Interfaces / Contracts
- `execute_openresponses_request` must not return `status=completed` for a
  mutating action request that produced zero tool execution evidence.
- Gateway error payloads should use the existing `gateway_failure` envelope so
  callers receive a consistent `BAD_GATEWAY` contract.
- Existing successful tool-loop behavior remains unchanged.

## Verification
- `cargo test -p tau-gateway openresponses_execution_handler -- --test-threads=1`
- `cargo test -p tau-gateway tier_pr_t4_tool_pipeline_executes_read_write_edit_sequence -- --test-threads=1`
