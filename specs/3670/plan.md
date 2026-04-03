# Plan: Issue #3670 - Recover read-only gateway timeouts and clear in-flight tools

## Approach
1. Extend pending-tool bookkeeping so timeout/error paths can synthesize
   terminal completion records for in-flight tools.
2. On attempt timeout, derive verifier state from observed traces instead of
   immediately forcing a terminal `gateway_timeout` block.
3. If the timed-out attempt is still verifier-`continue` and retry budget
   remains, route it through the existing bounded retry prompt path.
4. Keep retry exhaustion and non-recoverable failures fail-closed.

## Risks / Mitigations
- Risk: timeout recovery hides genuine provider stalls.
  Mitigation: retry only when the verifier still says `continue` and the bounded
  retry budget is not exhausted.
- Risk: synthesized timeout traces distort tool history.
  Mitigation: mark them explicitly as failed timeout completions and preserve
  the tool name/arguments in the persisted trace.

## Verification
- `cargo test -p tau-gateway regression_openresponses_timeout_after_read_only_tooling_retries_into_mutation -- --test-threads=1`
- `cargo test -p tau-gateway regression_openresponses_stream_timeout_finalizes_pending_tool_execution -- --test-threads=1`
- `cargo check -p tau-gateway`
- `cargo fmt --check`
