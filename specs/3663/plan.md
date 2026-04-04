# Plan: Issue #3663 - Persist gateway OpenResponses attempt payload traces for Ralph-loop debugging

## Approach
1. Add a small gateway-local attempt trace record type and persist it as JSONL
   under `.tau/gateway/openresponses/attempt-traces.jsonl`.
2. Write one record per attempt finish so the trace contains the normalized
   prompt, partial/final assistant text, tool evidence counts, and verifier or
   runtime failure result.
3. Reuse the existing JSONL append helper and keep the new diagnostics
   orthogonal to mission/session persistence.

## Proposed Design
### Trace record schema
- Persist a flat JSON object with:
  - `record_type`
  - `mission_id`
  - `session_key`
  - `response_id`
  - `attempt`
  - `started_unix_ms`
  - `finished_unix_ms`
  - `prompt`
  - `assistant_output`
  - `tool_execution_count`
  - `outcome_kind`
  - `verifier`
  - `completion`
  - `runtime_failure`
- Keep the schema normalized to the gateway loop rather than provider-specific
  wire payloads so it is stable across LLM clients.

### Write points
- Successful attempt:
  - after verifier/completion evaluation and before the next retry or final
    return
- Timeout/runtime failure:
  - right before mission state is marked blocked and the error is returned
- Use buffered stream text when available so action-oriented retries retain the
  hidden failed-attempt output that the TUI suppresses.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3663"
  change_surface:
    - symbol: "gateway attempt trace persistence"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "addition"
      current: "gateway missions persist summaries but no per-attempt trace"
      proposed: "persist one JSONL diagnostic record per attempt"
      compatibility: "safe"
      reason: "adds diagnostics without changing request/response contracts"
  overall_compatibility: "safe"
  approach:
    strategy: "Add opt-out-free gateway diagnostics alongside existing mission/session persistence"
    steps:
      - "define a trace path and append helper usage"
      - "persist traces on success, continue, timeout, and runtime failure"
      - "verify timed-out attempts still leave behind an inspectable record"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: trace records diverge from actual assistant text when buffered streaming
  is used.
  Mitigation: prefer buffered output for action-oriented retries and fall back to
  assistant message summaries otherwise.
- Risk: logging becomes noisy or too provider-specific.
  Mitigation: log the normalized gateway attempt payload and outcome, not raw
  transport envelopes.

## Verification
- `cargo test -p tau-gateway regression_openresponses_persists_attempt_trace_records_for_retry_timeout -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_action_retry_timeout_blocks_before_client_budget_exhaustion -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_retries_zero_tool_action_completion_until_tool_execution -- --test-threads=1`
