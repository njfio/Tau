# Spec: Break Read-Only Timeout Spiral In Gateway Action Retries

Status: Implemented
Issue: #3672

## Problem

Mutating gateway tasks can spend an entire outer-loop attempt on broad read-only exploration, time out, then immediately time out again on the retry because Tau carries forward bulky tool payloads and only adds a generic retry prompt. The loop is technically running, but it is not converging.

This is the read-only timeout spiral this issue must break.

## Scope

In:
- Gateway retry behavior after read-only timeout/continue attempts
- Retry prompt construction for mutation-required missions
- Regression coverage for compacted retry context

Out:
- New tool surfaces
- TUI UX beyond consuming the corrected gateway behavior

## Acceptance Criteria

### AC-1
Given a mutating gateway mission whose first attempt only performs read-only tooling and times out,
when Tau retries the mission,
then the retry request must preserve the original task, carry forward a compact observation summary, and avoid replaying the raw tool payloads from the failed attempt.

### AC-2
Given a mutating gateway mission whose verifier reports missing mutation evidence,
when Tau builds the retry prompt,
then the prompt must explicitly require a workspace-mutating tool next and discourage repeating broad exploration already captured in the observations.

### AC-3
Given the read-only timeout recovery path,
when the follow-up model response performs a mutation,
then the mission must complete successfully and the regression test must prove the corrected retry payload shape.

## Conformance Cases

- C-01: read-only timeout attempt is compacted into an observation summary before retry
- C-02: mutation-missing retry prompt explicitly demands a mutating tool next
- C-03: timeout recovery reaches successful mutation without replaying raw tool result payloads

## Success Signals

- The timeout-retry regression passes with explicit assertions on the retry request payload.
- Live retries stop carrying directory/file dumps from the failed attempt into the next request.
