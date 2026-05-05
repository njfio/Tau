# Spec: Issue #3769 - Harness operator action tones

Status: Implemented

## Problem Statement

The harness currently applies the same green treatment to all submit buttons in the preview. Secondary actions such as dry-run and benchmark execution should not look like approval actions because operators need to scan primary, destructive, secondary, and disabled actions quickly.

## Scope

In scope:

- Add durable action-tone markers for benchmark, approve, reject, secondary, and disabled actions.
- Scope green styling to the approval action only.
- Keep reject destructive styling and disabled apply styling intact.
- Give benchmark and dry-run actions a distinct non-approval tone.

Out of scope:

- Changing form actions, methods, routes, or preview submit guards.
- Adding icons or JavaScript behavior.
- Reworking proposal or benchmark data.

## Acceptance Criteria

### AC-1 Approval is visually distinct

Given `/ops/harness` renders, when operator actions are inspected, then `Approve` is the only approval-green action.

### AC-2 Secondary actions are not approval-green

Given `/ops/harness` renders, when benchmark and dry-run controls are inspected, then they expose non-approval action tones and non-approval styling.

### AC-3 Existing action contracts remain intact

Given `/ops/harness` renders, when forms and links are inspected, then action IDs, routes, methods, disabled apply state, and static preview guards remain intact.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | operator actions | inspect action tone markers | approve has approval tone and reject has destructive tone |
| C-02 | AC-2 | Functional | benchmark and dry-run controls | inspect style contract | secondary actions do not use approval-green styling |
| C-03 | AC-3 | Functional | harness route | run existing action tests | form routes, methods, disabled apply, and preview guard remain green |

## Success Metrics / Observable Signals

- Dashboard UI tests prove action-tone markers and scoped styling.
- Existing static preview form guard tests remain green.
- Browser preview shows benchmark/dry-run as non-approval actions.
- No new dependency is introduced.
