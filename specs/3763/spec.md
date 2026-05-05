# Spec: Issue #3763 - Harness proof evidence and log wrapping

Status: Implemented

## Problem Statement

The compact harness dashboard now fits, but proof evidence and terminal text still require horizontal scrolling at ordinary desktop preview width. Operators need tool evidence, operator logs, and the TUI companion to remain readable inside their panes without hidden right-side content.

## Scope

In scope:

- Compact the proof tool-evidence table at desktop preview widths.
- Hide the low-value artifact column only in the compact proof evidence view.
- Wrap tool evidence cell text instead of forcing horizontal table scroll.
- Wrap operator log and TUI preformatted output while preserving line breaks.
- Preserve all existing IDs, forms, routes, and state-backed data.

Out of scope:

- Changing tool evidence data.
- Adding client-side table expansion, column pickers, or resizing.
- Changing gateway behavior.

## Acceptance Criteria

### AC-1 Tool evidence fits the proof pane

Given `/ops/harness` renders at the current desktop preview width, when the tool evidence table is visible, then it uses compact fixed-layout styling and does not require horizontal pane scrolling.

### AC-2 Terminal logs wrap in place

Given operator log and TUI output contain long command lines, when compact desktop styles apply, then the text wraps inside the pane and preserves line breaks.

### AC-3 Compact behavior is testable and non-invasive

Given the route renders, when the style contract is inspected, then compact evidence/log markers are present and existing harness layout tests remain green.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness proof route | inspect rendered CSS | tool evidence table compact markers are present |
| C-02 | AC-2 | Functional | harness log panes | inspect rendered CSS | operator log and TUI use pre-wrap markers |
| C-03 | AC-3 | Functional | harness route | run dashboard UI tests | existing harness contract tests remain green |

## Success Metrics / Observable Signals

- Dashboard UI tests prove compact proof/log markers.
- Browser geometry reports no horizontal overflow in tool evidence, operator log, or TUI pre blocks at 1371px.
- Existing gateway-backed harness route integration remains green.
- No new dependency is introduced.
