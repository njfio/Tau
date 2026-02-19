# M96 - Spacebot G2 Context Compaction (Phase 2)

Milestone: GitHub milestone `M96 - Spacebot G2 Context Compaction (Phase 2)`

## Objective
Extend the existing G2 tiered compaction implementation by making compaction policy profile-configurable and wiring explicit context-pressure monitor semantics into runtime compaction decisions.

## Scope
- Add profile policy fields for context compaction thresholds and retention percentages.
- Wire profile policy values into local runtime/training `AgentConfig` construction.
- Keep compaction-tier behavior deterministic with conformance coverage for policy-fed values.

## Out of Scope
- Background async LLM compaction workers.
- Memory extraction/persistence during compaction.
- New message-role type for compaction summaries.

## Issue Hierarchy
- Epic: #2559
- Story: #2560
- Task: #2561
- Subtask: #2562

## Exit Criteria
- Task #2561 ACs pass with mapped conformance tests.
- Subtask #2562 evidence includes RED/GREEN, scoped gates, and live validation summary.
- `tasks/spacebot-comparison.md` G2 checklist reflects delivered phase-2 slice.
