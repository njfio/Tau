# M333 - Interactive gateway action-tool enforcement

Status: Active

## Context
The gateway-backed interactive TUI can currently accept an action-oriented
coding/build request, return a confident plain-text intent reply, and complete
the turn without any tool execution evidence. Operators see a successful answer
even though no files were created, no commands ran, and the Tools panel stays
empty. This milestone hardens the OpenResponses execution path so mutating
action requests fail closed unless the runtime actually attempted tool work.

## Issue Hierarchy
- Story: [#3651](https://github.com/njfio/Tau/issues/3651) Fail interactive
  gateway action turns that produce no tool evidence
- Task: [#3652](https://github.com/njfio/Tau/issues/3652) Retry mutating gateway
  turns when the model promises work without using tools

## Scope
- Detect action-oriented mutating requests in the gateway OpenResponses path
- Track per-turn tool execution evidence from the agent event stream
- Reject zero-tool completions for mutating action requests with a clear gateway
  error
- Retry zero-tool mutating action turns with corrective feedback before failing
  closed
- Add regression coverage around the gateway execution handler

## Exit Criteria
- Mutating interactive requests do not complete successfully when the model only
  emits plain intent text and performs zero tool executions
- Mutating interactive requests automatically retry with corrective feedback
  before the gateway gives up
- Non-action or conversational requests still complete normally without forcing
  tool use
- Gateway tests cover the zero-tool failure path and a non-action control path

## Delivery Notes
- Reuse the existing gateway action-token classifier instead of inventing a
  second heuristic surface
- Keep the behavior change confined to the gateway-backed interactive execution
  path and its tests
