# Spec: Issue #3765 - Harness compact evidence column priority

Status: Implemented

## Problem Statement

The proof evidence table now fits the pane, but runtime values can still wrap at compact desktop width because low-priority identifiers consume column space. Operators need the evidence table to prioritize tool, plan node, runtime, and status so timing and state remain readable at a glance.

## Scope

In scope:

- Hide low-value call IDs in the compact proof evidence view.
- Keep runtime and status columns readable without digit-level wrapping.
- Preserve all tool evidence rows, data attributes, and the wide-screen evidence table.
- Keep the artifact column hidden at compact width from the prior proof wrapping slice.

Out of scope:

- Removing call ID or artifact data from the wide layout.
- Adding table expansion controls.
- Changing proof data, gateway state, or mission semantics.

## Acceptance Criteria

### AC-1 Compact evidence prioritizes readable proof fields

Given `/ops/harness` renders at compact desktop width, when tool evidence is inspected, then call ID and artifact columns are hidden while tool, plan node, runtime, and status remain visible.

### AC-2 Runtime and status do not split digits

Given runtime and status values render in the compact evidence table, when compact styles apply, then those columns use nowrap styling and fixed widths.

### AC-3 Evidence behavior is testable and non-invasive

Given the route renders, when the style contract is inspected, then compact column-priority markers are present and existing harness route tests remain green.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | compact proof evidence table | inspect CSS markers | call ID and artifact columns are hidden |
| C-02 | AC-2 | Functional | runtime/status cells | inspect CSS markers | runtime and status columns use nowrap fixed-width styling |
| C-03 | AC-3 | Functional | harness route | run dashboard UI tests | existing harness contracts remain green |

## Success Metrics / Observable Signals

- Dashboard UI tests prove compact evidence column-priority markers.
- Browser geometry shows no horizontal overflow and no runtime/status button-like text overflow at 1371px.
- Existing gateway-backed harness route integration remains green.
- No new dependency is introduced.
