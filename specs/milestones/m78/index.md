# M78 - Spacebot G7 Memory Lifecycle (Phase 3)

Milestone: GitHub milestone `M78 - Spacebot G7 Memory Lifecycle (Phase 3)`

## Objective

Deliver the next production slice of `tasks/spacebot-comparison.md` gap `G7`
by adding near-duplicate lifecycle handling and heartbeat-triggered lifecycle
maintenance execution.

## Scope

- Extend lifecycle maintenance to detect near-duplicate active memories.
- Soft-forget non-canonical duplicates using deterministic policy.
- Wire runtime heartbeat to execute lifecycle maintenance against configured
  memory store roots.
- Surface lifecycle maintenance diagnostics in heartbeat reason codes and
  cycle reports.
- Add conformance and regression tests for duplicate + heartbeat paths.

## Out of Scope

- Embedding-model dependency changes (FastEmbed/Candle) for this slice.
- UI/operator screens for lifecycle internals.
- Cross-workspace deduplication beyond current store boundaries.

## Linked Hierarchy

- Epic: #2458
- Story: #2459
- Task: #2460
- Subtask: #2461
