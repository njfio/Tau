# M98 - Spacebot G2 Context Compaction (Phase 4)

Milestone: GitHub milestone `M98 - Spacebot G2 Context Compaction (Phase 4)`

## Objective
Close remaining G2 compaction gaps by persisting compaction summaries as first-class entries and extracting/saving memories during warn/aggressive compaction flow.

## Scope
- Persist generated compaction summaries as explicit compaction entries in session history.
- Extract memory candidates from compaction summaries and save them through existing memory pathways.
- Keep deterministic fallback when extraction or persistence fails.
- Conformance/regression tests for extraction/persistence behavior.

## Out of Scope
- New cross-session compactor/cortex process architecture.
- LLM-driven semantic memory graph enhancements beyond current memory tooling.
- Transport/channel behavior changes.

## Issue Hierarchy
- Epic: #2570
- Story: #2571
- Task: #2572
- Subtask: #2573

## Exit Criteria
- Compaction summaries persist as explicit session entries.
- Warn/aggressive compaction path attempts memory extraction/save deterministically.
- Evidence package includes conformance, mutation, and live validation outcomes.
