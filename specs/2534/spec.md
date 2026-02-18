# Spec #2534 - Epic: G15 profile routing closure (phase 2)

Status: Reviewed

## Problem Statement
G15 remains open because role-route model overrides are implemented only for plan-first orchestrator attempts. Profile-level process routing, task overrides, and complexity-aware model selection are still missing.

## Acceptance Criteria
### AC-1 closure scope
Given M92 closes, when implementation is complete, then remaining G15 checklist items for routing schema, task overrides, complexity scoring, and dispatch-time application are implemented and verified.

### AC-2 evidence completeness
Given this epic closes, when reviewed, then linked story/task/subtask specs include AC mapping, RED/GREEN evidence, mutation results, and live validation.

## Scope
In scope:
- Drive #2535/#2536/#2537 lifecycle to completion.

Out of scope:
- Unrelated roadmap items.

## Conformance Cases
- C-01 (AC-1): `spec_2536_c01_profile_defaults_parse_routing_fields`
- C-02 (AC-1): `spec_2536_c02_prompt_complexity_and_task_override_select_model`
- C-03 (AC-1): `spec_2536_c03_dispatch_uses_scoped_model_override_and_restores_baseline`
- C-04 (AC-2): PR includes complete tier matrix + RED/GREEN + mutation + live validation package

## Success Metrics
- Task #2536 merged with ACs green.
- Milestone #92 closed with linked artifacts and evidence.
