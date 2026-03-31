# Spec: Issue #3655 - Implement the first Tau Ralph supervisor loop slice for gateway missions

Status: Reviewed

## Problem Statement
`#3651` and `#3652` taught the gateway how to reject or retry zero-tool action
turns, but that behavior still lives as transient request-local control flow.
Tau does not yet create a durable mission object for the outer loop, so an
operator cannot inspect which mission is running, why the loop continued, which
verifier/back-pressure rule fired, or what session the work is linked to. The
first implementation slice toward the Tau Ralph architecture should make the
gateway persist a real mission-supervisor record around its outer retries while
remaining compatible with current prompt/session flows.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/request_translation.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3655/spec.md`
- `specs/3655/plan.md`
- `specs/3655/tasks.md`

Out of scope:
- new public gateway mission-control endpoints or TUI mission panes
- full verifier ecosystem beyond the current action-tool-evidence contract
- changing the inner `tau-agent-core` tool loop contract
- background/unbounded autonomous execution outside a single gateway request

## Acceptance Criteria
### AC-1 Gateway requests resolve a canonical mission id without breaking session compatibility
Given a gateway OpenResponses request,
when metadata includes `mission_id`,
then the runtime uses that sanitized mission id for durable mission state while
still using the existing session compatibility path for chat/session storage.

And given a gateway OpenResponses request without `mission_id`,
when the request is translated,
then the runtime derives an implicit single-mission id from the resolved
session key so legacy prompt/session flows continue to work unchanged.

### AC-2 The outer supervisor loop persists verifier-backed iteration history
Given a gateway mission whose action-oriented request enters the outer retry
loop,
when each outer iteration completes,
then the gateway persists mission-supervisor state that records the mission id,
linked session key, current status, and an ordered iteration history including
the verifier/back-pressure reason code and observed tool-execution evidence for
that iteration.

### AC-3 Successful completion marks the mission completed with durable linkage
Given a gateway mission whose outer loop eventually satisfies the current
verifier contract,
when the request completes successfully,
then the persisted mission state records a terminal `completed` status, the
linked session key, the response id, the latest verifier result, and the final
successful iteration count.

### AC-4 Retry exhaustion or gateway failure marks the mission blocked
Given a gateway mission whose outer loop exhausts the current verifier retry
budget or terminates with a gateway runtime failure,
when the request fails closed,
then the persisted mission state records a terminal `blocked` status and the
latest verifier/back-pressure reason explaining why the mission did not
complete.

## Conformance Cases
- C-01 / AC-1 / Unit:
  translate a request containing both `metadata.session_id` and
  `metadata.mission_id`, and verify the translated request preserves the
  session key while resolving the explicit mission id.
- C-02 / AC-1 / Unit:
  translate a request without `mission_id`, and verify the implicit mission id
  matches the sanitized session key.
- C-03 / AC-2, AC-3 / Regression:
  submit an action request whose first outer attempt produces zero tool
  evidence and whose retry executes a tool, then verify the persisted mission
  file records both iterations and ends in `completed`.
- C-04 / AC-2, AC-4 / Regression:
  submit an action request whose outer attempts all produce zero tool evidence,
  then verify the persisted mission file records retry continuation and ends in
  `blocked` with the retry-exhausted verifier reason.

## Success Metrics / Observable Signals
- Every gateway request now has one canonical mission id, even on the legacy
  single-session path
- Gateway outer-loop decisions are inspectable on disk instead of disappearing
  into request-local control flow
- The first Tau Ralph slice produces a durable mission object that links goal,
  verifier outcomes, response id, and session lineage

## Files To Touch
- `specs/3655/spec.md`
- `specs/3655/plan.md`
- `specs/3655/tasks.md`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/request_translation.rs`
- `crates/tau-gateway/src/gateway_openresponses/types.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
