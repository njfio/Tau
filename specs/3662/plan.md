# Plan: Issue #3662 - Bound no-tool gateway retry attempts so TUI does not time out first

## Approach
1. Keep the existing verifier contract and retry count, but shorten the timeout
   budget for later action retries where the prior attempts already proved the
   model is stalling without tools.
2. Compute the retry timeout from the configured turn timeout so tests can drive
   it with small values while production still gets a reasonable bounded cap.
3. Preserve current success behavior for retries that actually execute tools.

## Proposed Design
### Retry timeout policy
- Introduce a helper in `openresponses_execution_handler.rs` that resolves the
  per-attempt timeout:
  - first attempt keeps the configured `turn_timeout_ms`
  - later action retries use a reduced timeout derived from `turn_timeout_ms`
    with a hard upper cap
- Apply the bounded timeout only when `requires_tool_evidence` is true and
  `retry_attempt > 0`.

### Failure behavior
- Reuse the existing gateway timeout failure path so mission state stays
  consistent:
  - persist tool history
  - record an iteration with the timeout verifier bundle
  - mark the mission blocked
  - return a timeout response

## Compatibility Assessment
```yaml
implementation_strategy:
  task: "3662"
  change_surface:
    - symbol: "gateway action retry timeout policy"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "every retry attempt can use the full turn timeout"
      proposed: "later action retries use a bounded retry timeout"
      compatibility: "safe"
      reason: "narrows failure latency for an already failing no-tool path"
  overall_compatibility: "safe"
  approach:
    strategy: "Bound later no-tool action retries without changing success semantics"
    steps:
      - "derive a shorter retry timeout from turn_timeout_ms"
      - "apply it only to later action retries"
      - "cover timeout block persistence with a regression"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: retry timeout becomes too aggressive and cuts off successful tool use.
  Mitigation: apply the bound only on later action retries, not the first
  attempt.
- Risk: tests become flaky due to timing.
  Mitigation: use a deterministic delayed fixture client and small time budgets.

## Verification
- `cargo test -p tau-gateway regression_openresponses_action_retry_timeout_blocks_before_client_budget_exhaustion -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_persists_blocked_mission_state_for_retry_timeout -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_retries_zero_tool_action_completion_until_tool_execution -- --test-threads=1`
