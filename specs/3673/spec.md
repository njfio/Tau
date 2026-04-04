# Spec: Force Tool-Required Retry Turns In Ralph-Loop Recovery

Status: Reviewed

## Problem

Gateway Ralph-loop retries can know that tool or mutation evidence is required, but the underlying agent request still uses automatic tool choice. CLI provider adapters also ignore `tool_choice` semantics and render the same soft contract for both auto and required turns. This allows recovery turns to stall instead of immediately emitting a tool call.

## Scope

In:
- Agent runtime support for overriding tool choice on a turn/attempt
- Gateway retry path using required tool choice for action retries
- CLI provider adapters honoring required tool choice in rendered prompt contracts
- Regression coverage for required-tool retry requests

Out:
- New tool surfaces
- Non-CLI provider behavior changes beyond existing tool-choice support

## Acceptance Criteria

### AC-1
Given a gateway retry turn where tool evidence is already known to be required,
when Tau builds the outbound chat request,
then the request must use `ToolChoice::Required` instead of `ToolChoice::Auto`.

### AC-2
Given a CLI provider adapter receives a request with `ToolChoice::Required`,
when it renders the execution prompt,
then the prompt contract must explicitly require a textual tool-call JSON response rather than allowing plain assistant text.

### AC-3
Given a mutating gateway mission that enters a retry after missing mutation evidence,
when the retry request is captured in regression tests,
then the captured request must show required tool choice and the mission must still recover successfully.

## Conformance Cases

- C-01: gateway retry request captures `tool_choice = required`
- C-02: codex/claude/gemini CLI prompt contracts tighten for required tool choice
- C-03: mutation retry regression still completes successfully under required tool choice

## Success Signals

- Gateway retry tests assert `ToolChoice::Required` on captured retry requests.
- CLI provider tests assert the stricter required-tool contract text.
