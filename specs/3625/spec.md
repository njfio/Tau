# Spec: Issue #3625 - Make MCP lifecycle tools explicit about unsupported runtime operations

Status: Implemented
Priority: P1
Milestone: M329
Parent: #3623

## Problem Statement
The MCP server currently exposes lifecycle-style tools for training and agent
orchestration that return success-shaped responses even though no live runtime
binding exists in `McpServerState`. That misleads callers into believing work
was accepted or scheduled when the server has not actually done anything.

## Scope
- Replace fake-success MCP lifecycle responses with explicit unsupported/runtime
  unavailable error contracts.
- Update MCP tool descriptions and generated MCP SDK docs so capability claims
  match runtime behavior.
- Add regression/conformance coverage for the new response contract.

## Out of Scope
- Wiring these tools to a real orchestration or training runtime.
- MCP skills parity with `tau-skills`.
- `plan_executor` alignment work.

## Acceptance Criteria
- AC-1: `tau.training_trigger` no longer returns a success-shaped response when
  no training runtime is wired in the MCP server context.
- AC-2: `tau.agent_spawn`, `tau.agent_status`, and `tau.agent_cancel` no longer
  return success-shaped or ambiguous placeholder responses when no orchestration
  runtime is wired in the MCP server context.
- AC-3: Tool descriptions/docs clearly state the runtime requirement or current
  unsupported behavior.
- AC-4: Tests prove the unsupported/runtime-unavailable response contract and
  prevent reintroduction of fake-success behavior.

## Conformance Cases
- C-01 (AC-1, regression): `tau.training_trigger` returns `isError=true` with a
  stable unsupported/runtime-unavailable contract instead of `"accepted"`.
- C-02 (AC-2, regression): `tau.agent_spawn`, `tau.agent_status`, and
  `tau.agent_cancel` each return `isError=true` with an explicit
  unsupported/runtime-unavailable contract instead of fake success or
  `"unknown"`.
- C-03 (AC-3, conformance): MCP tool descriptions/docs mention the runtime
  dependency or unsupported semantics instead of implying guaranteed execution.
- C-04 (AC-4, conformance): scoped `tau-tools` tests and MCP SDK doc tests stay
  green with the updated contract.

## Success Signals
- MCP callers can reliably detect that these lifecycle operations are not
  executable in the current server context.
- No exposed lifecycle MCP tool reports acceptance for work that was not
  actually scheduled.
- Tool descriptions/docs no longer overclaim runtime capabilities.
