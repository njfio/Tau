# Plan: Issue #2425 - G12 phase-2 SkipTool implementation and validation

## Approach
1. Add RED conformance tests in `tau-tools`, `tau-agent-core`, `tau-runtime`, and `tau-gateway`
   for skip tool registration, payload shape, loop termination, and output suppression.
2. Implement `SkipTool` in `tau-tools` and register it in built-in registry/name catalog.
3. Extend `tau-agent-core` run loop to detect successful skip directives and terminate without a
   follow-up model completion turn.
4. Add shared skip-directive extraction helper in `tau-agent-core`, then consume it in
   `tau-runtime` output rendering and gateway/session reply collection.
5. Run scoped verify gates and targeted regression for `/tau skip` command behavior.

## Affected Modules
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/tests.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-runtime/src/runtime_output_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/session_runtime.rs`
- `crates/tau-multi-channel/src/multi_channel_runtime/tests.rs` (regression verification run)

## Risks / Mitigations
- Risk: false-positive suppression from unrelated tool outputs.
  - Mitigation: require `tool_name == "skip"` and explicit suppression marker in parsed payload.
- Risk: run-loop control regression in tool-heavy flows.
  - Mitigation: preserve existing tool execution path and add dedicated integration test proving one
    model completion turn when skip is used.
- Risk: hidden fallback text leaks into user output.
  - Mitigation: update collectors to check skip directive before fallback rendering.

## Interfaces / Contracts
- New built-in tool contract:
  - name: `skip`
  - args: `{ "reason"?: string }`
  - success payload: `{ "skip_response": true, "reason": string, "reason_code": "skip_suppressed" }`
- New helper in `tau-agent-core`:
  - `extract_skip_response_reason(messages: &[Message]) -> Option<String>`

## ADR
- Not required: no new dependencies, wire protocol, or architecture-level replacement.
