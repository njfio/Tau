# M61 — Spacebot G5 Typed Memories + Importance Scoring (Foundation Slice)

Milestone: [GitHub milestone #61](https://github.com/njfio/Tau/milestone/61)

## Objective

Implement `tasks/spacebot-comparison.md` gap `G5` foundation in Tau memory runtime:
typed memory classification and importance scoring integrated into memory write,
persisted records, and search ranking output.

## Scope

- Add `MemoryType` taxonomy aligned to G5:
  `identity`, `goal`, `decision`, `todo`, `preference`, `fact`, `event`,
  `observation`.
- Add per-type default importance values.
- Extend `memory_write` to accept optional `memory_type` and `importance`.
- Persist type/importance metadata on records.
- Include type/importance in read/search tool outputs.
- Apply importance influence in memory search ranking.

## Out of Scope

- Memory graph relations (`G6`).
- Memory lifecycle decay/pruning/dedup (`G7`).
- Bulk memory ingestion (`G9`).

## Linked Hierarchy

- Epic: #2369
- Story: #2370
- Task: #2371
- Subtask: #2372
