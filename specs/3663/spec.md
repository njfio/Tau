# Spec: Issue #3663 - Persist gateway OpenResponses attempt payload traces for Ralph-loop debugging

Status: Implemented

## Problem Statement
The gateway Ralph loop currently persists mission summaries, verifier outcomes,
and top-level session messages, but it does not persist the actual per-attempt
normalized prompts, assistant outputs, or terminal failure payloads. When a
request stalls or times out, operators cannot reconstruct what each retry
attempt actually sent to the model or what came back before the loop blocked.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3663/spec.md`
- `specs/3663/plan.md`
- `specs/3663/tasks.md`
- `specs/milestones/m334/index.md`

Out of scope:
- changing TUI rendering behavior
- changing retry-count policy or verifier semantics
- logging raw provider wire payloads beyond the normalized gateway attempt view

## Acceptance Criteria
### AC-1 Every gateway attempt persists a durable trace record
Given the gateway executes an OpenResponses turn,
when each attempt starts or finishes,
then it appends a trace record under the gateway state dir that captures the
attempt lifecycle.

### AC-2 Trace records include the attempt payload and outcome
Given a gateway attempt trace record is persisted,
when an operator inspects it,
then it includes the mission id, session key, response id, attempt number,
normalized prompt, assistant summary or buffered output, tool execution count,
verifier outcome, and completion/runtime failure details when present.

### AC-3 Timed-out and runtime-failed attempts remain inspectable
Given a retry attempt times out or the gateway runtime fails mid-loop,
when the gateway returns the blocked failure,
then the corresponding attempt trace still exists on disk with the failure
reason and partial output captured so far.

### AC-4 Existing mission/session behavior remains intact
Given the gateway persists attempt traces,
when successful and failing Ralph-loop requests run,
then existing mission and session persistence semantics remain unchanged apart
from the added diagnostics.

## Conformance Cases
- C-01 / AC-1 / Regression:
  execute an action-oriented retry flow and verify an attempt-trace JSONL file is
  created under `.tau/gateway/openresponses/`.
- C-02 / AC-2 / Regression:
  parse the persisted trace records and assert they contain mission/session
  identity, prompt text, assistant text, tool counts, and verifier outcome.
- C-03 / AC-3 / Regression:
  drive a bounded retry timeout and verify the final persisted trace record has
  `outcome_kind=runtime_failure` with the timeout reason.
- C-04 / AC-4 / Functional:
  keep existing mission/session retry regressions green while the new traces are
  written.

## Success Metrics / Observable Signals
- Operators can inspect `.tau/gateway/openresponses/attempt-traces.jsonl` after a
  failed request and see what each retry actually did.
- Timeout/debug sessions are no longer opaque “nothing happened” failures.
- Existing mission/session artifacts continue to load and pass existing tests.

## Files To Touch
- `specs/3663/spec.md`
- `specs/3663/plan.md`
- `specs/3663/tasks.md`
- `specs/milestones/m334/index.md`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
