# Plan: Break Read-Only Timeout Spiral In Gateway Action Retries

## Approach

1. Detect the mutation-missing retry path after a read-only attempt.
2. Rewrite the failed attempt context so the next request keeps:
   - the original task
   - a compact observation summary derived from observed tool traces
3. Drop raw tool payload messages from the failed attempt before retry.
4. Strengthen the retry prompt with mutation-first directives.
5. Extend the existing timeout-retry regression to assert the new payload shape.

## Current Code Reality

- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
   already retries when verifier status is `Continue`, including timeout attempts
   that observed tool execution.
- The timeout branch still feeds `build_gateway_action_retry_prompt` only the
   verifier bundle after stripping assistant messages, so the next prompt can
   lack a compact explanation of prior read-only evidence.
- The `request_payload` / `response_payload` mission trace fields from #3671
   give this stage an observable place to assert retry payload shape.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks

- Over-aggressive context compaction could hide useful details for retries that genuinely need prior tool output.

## Mitigation

- Limit compaction to the mutation-missing retry path.
- Preserve the original task explicitly.
- Summarize observed read-only tools in a compact bulletin instead of dropping all context.
