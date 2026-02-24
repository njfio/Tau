# Spec: Issue #3558 - Interactive start marker must expose both timeout domains

Status: Implemented

## Problem Statement
Interactive TTY runs emit a start line that can report `timeout_ms=0` even when
request-timeout enforcement is active (for example `--request-timeout-ms 45000`).
This obscures the effective timeout behavior and makes troubleshooting needlessly
difficult.

## Scope
In scope:
- Include both `turn_timeout_ms` and `request_timeout_ms` in interactive start
  progress output.
- Ensure interactive runtime wiring passes request timeout through to marker
  emission.
- Update tests and operator docs to match the output contract.

Out of scope:
- Retry strategy changes.
- Dashboard render changes.
- Provider transport changes unrelated to marker output.

## Acceptance Criteria
### AC-1 Start marker contains explicit timeout fields
Given an interactive TTY turn begins,
when the progress tracker emits the start marker,
then the line includes both `turn_timeout_ms=<n>` and
`request_timeout_ms=<n>`.

### AC-2 Interactive runtime wiring carries request timeout
Given local runtime is configured with CLI request timeout,
when interactive config is constructed and used,
then request timeout value reaches the progress tracker unchanged.

### AC-3 Docs and tests reflect the new marker contract
Given operator-facing docs and runtime-loop line-contract tests,
when updated for this change,
then they assert/describe the dual-timeout start marker format.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Unit | start-line formatter inputs `0` and `45000` | format start line | output includes both timeout fields |
| C-02 | AC-2 | Functional | interactive runtime config built from CLI defaults | dispatch prompt | progress tracker receives configured request timeout |
| C-03 | AC-3 | Regression | README/operator guide and runtime-loop tests | run test/docs checks | examples match emitted marker contract |

## Success Metrics / Observable Signals
- Operators can immediately identify whether a timeout failure came from
  request timeout vs. turn timeout settings.
- Runtime-loop tests enforce the marker string contract.
- No non-TTY noise regressions.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `runtime_loop::tests::unit_interactive_turn_progress_line_contract_is_stable` now asserts `interactive.turn=start turn_timeout_ms=0 request_timeout_ms=45000`. |
| AC-2 | ✅ | `startup_local_runtime` sets `InteractiveRuntimeConfig.request_timeout_ms` from `cli.request_timeout_ms.max(1)`; verified by `regression_spec_3555_c01_run_local_runtime_uses_cli_request_timeout_for_agent`. |
| AC-3 | ✅ | Updated marker format in `README.md` and `docs/guides/operator-deployment-guide.md`; unit test contract updated in `runtime_loop.rs`. |
