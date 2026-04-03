# Spec: Issue #3731 - Hoist action history I/O out of per-attempt loop

Status: Reviewed

## Problem Statement
The gateway OpenResponses retry loop currently reloads the full action-history
JSONL on each retry to build the learning bulletin, and it also reloads and
rewrites the full store on every attempt when persisting tool traces. This
creates repeated disk I/O on the hot request path even though the request only
needs one action-history snapshot plus in-memory appends until the request
finishes.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/learning_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `specs/3731/spec.md`
- `specs/3731/plan.md`
- `specs/3731/tasks.md`

Out of scope:
- changing learning bulletin semantics
- changing action-history record contents
- changing mission persistence semantics

## Acceptance Criteria
### AC-1 Request execution loads action history only once
Given a gateway OpenResponses request with retries,
when the request starts,
then the action-history store is loaded once before the retry loop instead of
being reloaded on every attempt.

### AC-2 Attempt persistence appends in memory without per-attempt disk writes
Given a retry attempt with observed tool traces,
when the gateway persists attempt tool history,
then it appends the records to an in-memory action-history store instead of
performing a load+save cycle for that attempt.

### AC-3 Learning bulletin rendering can use an already-loaded store
Given the request already holds an in-memory action-history store,
when the gateway refreshes the learning bulletin inside the retry loop,
then it renders from the in-memory store without re-reading the JSONL file.

### AC-4 Action history still persists at request completion
Given a gateway request finishes with success or failure,
when the handler exits,
then the accumulated action-history records are saved once so the next request
sees the same bulletin/learning data as before.

## Conformance Cases
- C-01 / AC-2 / Unit: appending gateway action-history records mutates an
  in-memory `ActionHistoryStore` and does not create the JSONL file by itself.
- C-02 / AC-3 / Unit: rendering a learning bulletin from an in-memory store
  matches the existing disk-backed rendering path.
- C-03 / AC-1 / AC-4 / Functional: request execution loads the store before the
  retry loop and saves it once after the loop completes.

## Success Metrics / Observable Signals
- The request hot path avoids repeated full-file action-history load/save cycles.
- Learning bulletin content stays unchanged for the same store contents.
- Existing gateway behavior remains functionally equivalent with fewer I/O calls.
