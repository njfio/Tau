# Spec: Issue #3651 - Fail interactive gateway action turns that produce no tool evidence

Status: Reviewed

## Problem Statement
Gateway-backed interactive turns can currently accept a mutating action request,
return a plain assistant promise such as "I'll create the files...", and mark
the response completed even though no tool executed. In practice that means the
operator sees a successful coding/build reply while nothing happened on disk and
the TUI Tools panel remains empty. The gateway runtime is healthy and tools are
registered; the failure is that the execution path treats a zero-tool text
completion as success for a request that plainly required actions.

## Scope
In scope:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/milestones/m333/index.md`
- `specs/3651/spec.md`
- `specs/3651/plan.md`
- `specs/3651/tasks.md`

Out of scope:
- changing tool registration or gateway startup/bootstrap behavior
- retrying or auto-reprompting the model after a zero-tool completion
- changing the legacy non-gateway interactive path

## Acceptance Criteria
### AC-1 Mutating action requests fail closed without tool evidence
Given a gateway-backed OpenResponses request whose prompt is action-oriented and
mutating,
when the agent completes the turn with zero tool executions,
then the gateway responds with a non-success error instead of a completed
assistant message and the error explains that the runtime produced no tool
execution evidence for an action request.

### AC-2 Conversational requests still allow zero-tool completions
Given a gateway-backed OpenResponses request whose prompt is conversational or
informational rather than mutating,
when the agent completes the turn with zero tool executions,
then the gateway may still return a normal completed assistant response.

### AC-3 Real tool execution evidence still permits action completions
Given a gateway-backed OpenResponses request whose prompt is action-oriented and
mutating,
when at least one tool execution occurs during the turn,
then the gateway preserves the normal completed response path.

## Conformance Cases
- C-01 / AC-1 / Regression:
  submit an action request such as "create a Phaser game", script the model to
  return only plain assistant text, and observe `BAD_GATEWAY` with a zero-tool
  enforcement message.
- C-02 / AC-2 / Functional:
  submit a conversational request such as "explain what Phaser is", script the
  model to return plain assistant text, and observe `200 OK` plus the assistant
  response.
- C-03 / AC-3 / Regression:
  submit an action request, script at least one real tool call before the final
  assistant message, and observe `200 OK`.

## Success Metrics / Observable Signals
- The gateway no longer records successful completed action turns with zero tool
  evidence
- Operators receive an actionable gateway failure instead of a misleading
  "I'll do it" reply when nothing executed
- Gateway test coverage pins both the failure path and the conversational
  control path

## Files To Touch
- `specs/milestones/m333/index.md`
- `specs/3651/spec.md`
- `specs/3651/plan.md`
- `specs/3651/tasks.md`
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
