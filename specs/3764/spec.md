# Spec: Issue #3764 - Harness self-improvement action priority

Status: Implemented

## Problem Statement

The self-improvement review pane renders the proposal and policy details, but the approval controls sit below the conservative policy panel. Operators need the approve, reject, dry-run, diff, and gated apply controls to be reachable as the primary review workflow instead of being buried beneath secondary policy context.

## Scope

In scope:

- Move the self-improvement operator action controls above the conservative policy panel.
- Mark the action placement as a durable UI contract.
- Compact the action controls into a predictable grid that keeps buttons readable.
- Preserve all proposal routes, forms, IDs, approval gating, and audit data.

Out of scope:

- Changing self-improvement approval semantics.
- Making apply autonomous or bypassing approval.
- Adding new proposal data, modal flows, or client-side interaction state.
- Changing gateway behavior.

## Acceptance Criteria

### AC-1 Operator actions are first-class in the review pane

Given `/ops/harness` renders the self-improvement review pane, when the pane is inspected, then `Operator Actions` appears before the conservative policy section.

### AC-2 Approval controls stay complete and gated

Given the action section renders, when its controls are inspected, then approve, reject, dry-run, view-diff, and apply controls remain present with their existing routes and approval-required state.

### AC-3 Compact action layout is testable

Given the route renders, when the style contract is inspected, then action-priority and compact action-grid markers are present and existing harness tests remain green.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness self-improvement pane | inspect rendered HTML order | operator actions appear before conservative policy |
| C-02 | AC-2 | Functional | action controls | inspect rendered forms/buttons | approval, rejection, dry-run, diff, and apply controls remain intact and approval-gated |
| C-03 | AC-3 | Functional | harness route | inspect rendered CSS | action-priority grid markers are present |

## Success Metrics / Observable Signals

- Dashboard UI tests prove action priority, preserved controls, and compact action-grid styling.
- Browser geometry shows the operator action section is visible above the policy/audit context at 1371px desktop preview width.
- Existing harness and gateway-backed route tests remain green.
- No new dependency is introduced.
