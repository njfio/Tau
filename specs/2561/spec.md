# Spec #2561 - Task: implement profile compaction policy defaults + runtime wiring

Status: Reviewed
Priority: P0
Milestone: M96
Parent: #2560

## Problem Statement
`tau-agent-core` already supports tiered context compaction, but the policy values are not sourced from profile defaults. Operators cannot tune compaction thresholds/retention via profile-managed runtime config.

## Scope
- `tau-onboarding` profile defaults/schema for context compaction policy fields.
- Runtime wiring (`tau-coding-agent`, onboarding local runtime, training runtime) to feed those values into `AgentConfig`.
- Conformance/regression tests for parsing/backfill and runtime policy mapping.

## Out of Scope
- Background compaction worker orchestration.
- Compaction memory extraction/persistence.
- New transport behavior changes.

## Acceptance Criteria
- AC-1: Given profile defaults JSON omits compaction policy fields, when deserialized, then deterministic defaults are applied (80/85/95 thresholds and 70/50/50 retain values).
- AC-2: Given profile defaults provide explicit compaction policy values, when local runtime agent settings are built, then the resulting `AgentConfig` uses those exact values.
- AC-3: Given training runtime builds an agent executor, when settings are materialized, then compaction policy fields are populated deterministically and remain backward compatible.
- AC-4: Given legacy profile store entries without compaction policy fields, when loaded, then load succeeds with defaulted policy values.

## Conformance Cases
- C-01 (AC-1, unit): Parse `ProfileDefaults` JSON without compaction policy keys and assert default values.
- C-02 (AC-2, functional): Build local runtime agent with custom policy values and assert compaction behavior/derived config path reflects those values.
- C-03 (AC-3, unit): Verify training runtime settings include compaction policy values (default path).
- C-04 (AC-4, regression): Load legacy profile store fixture and assert defaulted compaction policy fields are present.

## Success Signals
- No profile parsing regressions for existing stores.
- Runtime compaction policy values become profile-configurable without changing default behavior.
- Scoped tests for onboarding/coding-agent/agent-core pass.
