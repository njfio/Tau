# Plan: Cut Off Read-Only Exploration Spirals And Widen Mutation Recovery Budget

## Approach

1. Introduce gateway-local helpers that classify tool executions as read-only
   versus mutating using the existing verifier semantics.
2. Create a per-attempt cooperative cancellation token and install it on the
   agent before `prompt_with_stream`.
3. In the event subscriber, count successful read-only tools for mutation-
   required attempts. Once a bounded threshold is reached with no successful
   mutating evidence, cancel the current attempt.
4. Teach the timeout/cancellation handling path to recognize this saturation
   cancellation as a verifier-driven `continue` outcome instead of a terminal
   runtime failure.
5. Increase the mutation-recovery retry timeout floor so retries have a
   realistic chance to emit the first mutating tool call.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/verifier_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks

- Cancelling too aggressively could stop legitimate discovery before mutation.
  Mitigation: require multiple successful read-only tools and only enable the
  policy for mutation-required missions.
- Timeout increase could lengthen failed retries too much.
  Mitigation: use a bounded floor rather than restoring the full turn timeout.

## Verification

- Gateway regression for read-only saturation cancellation and retry
- Gateway regression for widened retry timeout floor
- Existing mutation-retry regression remains green
