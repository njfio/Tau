# M71 - Spacebot G12 Skip Tool (LLM Tool Wiring)

Milestone objective: deliver the phase-2 G12 scope that adds a first-class `skip` tool in
the agent tool registry and runtime orchestration so a model can intentionally suppress one
outbound response while preserving auditable reason metadata.

## Scope
- Add `skip` as a built-in tool in `tau-tools`.
- Define structured skip tool input (`reason`) and result payload contract.
- Wire agent run-loop/runtime handling so a `skip` tool call suppresses outbound response.
- Propagate skip metadata into session/message logs for observability.
- Add conformance and regression tests for tool registration, parsing, and runtime behavior.

## Out of Scope
- Replacing existing `/tau skip` command path in `tau-multi-channel`.
- Provider/model routing changes.
- Reaction/file tooling from G13/G14.

## Exit Criteria
- Task issue `#2425` has accepted spec, implemented code, and passing verification.
- Tool registry and runtime behavior are covered by conformance tests.
- `fmt`, scoped `clippy`, and scoped tests pass for touched crates.
