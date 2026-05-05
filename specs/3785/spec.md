# Spec 3785: Review Queue Full-Label Readability

Status: Implemented

## Problem

The right self-improvement review pane shows all four learning/proposal items, but their labels are clipped mid-word in two-column pills. Operators cannot quickly distinguish the proposal queue items without guessing from truncated text.

## Acceptance Criteria

AC-1: Given `/ops/harness` is rendered in the desktop harness preview, when the Learning & Proposals queue is visible, then all four item labels are readable without horizontal text clipping.

AC-2: Given the queue is compacted for the right review pane, when the queue layout changes, then all four learning/proposal records remain visible before Operator Actions.

AC-3: Given proposal readability is improved, when the review pane is rendered, then the queue still preserves learning IDs and proposal IDs in DOM order for operator traceability.

## Scope

In scope:
- Tau Agent Harness self-improvement review queue HTML/CSS contract.
- Compact full-label queue layout.
- Conformance test plus browser geometry validation.

Out of scope:
- Changing proposal data, approval flow, policy cards, or audit rows.
- Gateway behavior.
- TUI behavior.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert Learning & Proposals declares full-label readability with no truncation budget.

C-02 maps AC-2: Render `/ops/harness` and assert the queue remains before Operator Actions while keeping all four rows visible.

C-03 maps AC-3: Render `/ops/harness` and assert LR-219, LR-220, PR-044, and PR-045 remain present in DOM order.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3785` passes.
- Browser geometry confirms zero Learning & Proposals label truncation and no document horizontal overflow.
- Existing harness regression tests remain green.
