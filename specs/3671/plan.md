# Plan: Issue #3671 - Add raw gateway payload tracing and reconcile TUI tool lifecycle state

## Approach
Current code reality: the TUI active-count symptom is partially fixed by
completion matching on tool name, but tool_call_id reconciliation remains open in
the reducer. Gateway mission iterations still persist summaries only;
request_payload and response_payload fields are not yet persisted on mission
iterations.

1. Add red regressions for payload-rich attempt tracing and TUI tool lifecycle
   reconciliation.
2. Extend gateway attempt trace records so they include structured outbound
   request and inbound response payload evidence for each attempt.
3. Change the TUI tool reducer from append-only lifecycle logging to
   start/completion reconciliation keyed by tool call identity.
4. Rerun the scoped gateway/TUI verification stack.

## Proposed Design
### Gateway attempt payload tracing
- Persist structured per-attempt request payload evidence, at minimum the
  normalized prompt plus the exact request messages/tool-result messages the
  gateway handed to the model for that attempt.
- Persist structured inbound response payload evidence, at minimum assistant
  text and/or assistant content blocks for the attempt result.
- Keep the existing summary fields for quick scanning, but treat them as
  derivative from the raw-ish payload fields.

### TUI tool lifecycle reconciliation
- Include `tool_call_id` in streamed tool start/completion events all the way to
  the TUI reducer.
- Track running tool entries by `tool_call_id`.
- On completion, update the matching running entry in place instead of pushing a
- second terminal row that leaves the original `Running` row active forever.

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3671"
  change_surface:
    - symbol: "gateway attempt trace persistence"
      location: "crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs"
      change_type: "modification"
      current: "lossy attempt summaries without request/response payload evidence"
      proposed: "structured attempt traces containing outbound/inbound payload fields"
      compatibility: "safe"
      reason: "additive trace fields improve observability without changing request semantics"
    - symbol: "TUI tool reducer"
      location: "crates/tau-tui/src/interactive/{gateway_client,app,tools}.rs"
      change_type: "modification"
      current: "append-only tool lifecycle entries keep stale Running rows active"
      proposed: "tool lifecycle is reconciled by tool_call_id"
      compatibility: "safe"
      reason: "corrects UI state without changing external protocol semantics beyond additive identifiers"
  overall_compatibility: "safe"
```

## Risks / Mitigations
- Risk: payload tracing grows trace files too quickly.
  Mitigation: log only per-attempt request/response evidence, not every internal
  object or provider transport detail.
- Risk: changing the TUI reducer breaks existing transcript rendering.
  Mitigation: add focused reducer tests plus the existing SSE/TUI gateway tests.

## Verification
- `cargo test -p tau-gateway regression_openresponses_attempt_traces_capture_request_and_response_payloads -- --test-threads=1`
- `cargo test -p tau-tui interactive::app_gateway_tests -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_stream_timeout_finalizes_pending_tool_execution -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs crates/tau-gateway/src/gateway_openresponses/tests.rs crates/tau-tui/src/interactive/gateway_client.rs crates/tau-tui/src/interactive/app.rs crates/tau-tui/src/interactive/tools.rs crates/tau-tui/src/interactive/app_gateway_tests.rs`
