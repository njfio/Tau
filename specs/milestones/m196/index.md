# M196 - Tau Ops Dashboard PRD Phase 3I (Memory Graph Node Size Contracts)

## Context
Implements Tau Ops Dashboard PRD memory-graph checklist contract:
- `2088` "Node size reflects importance"

for `/ops/memory-graph`.

## Linked Issues
- Epic: #3074
- Story: #3071
- Task: #3070

## Scope
- Deterministic node-size markers on `/ops/memory-graph` node rows.
- Gateway importance normalization and size-bucket derivation.
- Deterministic numeric size marker values for SSR validation.
- Regression safety for existing memory-graph and memory explorer contracts.

## Out of Scope
- Node color semantics (`2089`).
- Edge style semantics (`2090`) and graph interactions (`2091`-`2095`).
- New memory relation/ranking algorithms.
