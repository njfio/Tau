# M198 - Tau Ops Dashboard PRD Phase 3K (Memory Graph Edge Style Contracts)

## Context
Implements Tau Ops Dashboard PRD memory-graph checklist contract:
- `2090` "Edge style reflects relation type"

for `/ops/memory-graph`.

## Linked Issues
- Epic: #3080
- Story: #3081
- Task: #3082

## Scope
- Deterministic edge-style markers on `/ops/memory-graph` edge rows.
- Stable relation-type to style-token/dasharray mapping.
- UI and gateway conformance tests for style marker rendering.
- Regression safety for existing memory-graph and memory-explorer contracts.

## Out of Scope
- Node-style contracts (`2088`, `2089`) already implemented.
- Interactive graph behavior (`2091`-`2095`).
- New dependencies or protocol changes.
