# Issue 3791: Harness Proposal Detail Does Not Vertically Clip

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The self-improvement proposal detail card uses a contained hidden-overflow
budget but its content is slightly taller than the available box. The result is
a clipped proposal proof row region even though the card claims all seven
summary rows are visible.

## Scope

In scope:
- Keep the proposal detail before the audit history in the first review window.
- Preserve all seven proposal detail rows.
- Eliminate hidden vertical overflow in the proposal detail card at the desktop
  harness preview viewport.

Out of scope:
- Changing proposal data or approval semantics.
- Reordering the learning queue, proposal detail, or audit log.
- Expanding the gateway/channel surfaces.

## Acceptance Criteria

AC-1: Given the `/ops/harness` review pane renders, when the proposal detail
card is displayed, then all seven detail rows remain present.

AC-2: Given the proposal detail card uses contained overflow, when the desktop
preview viewport is measured, then the card content height fits its client
height without hidden vertical clipping.

AC-3: Given the proposal detail card is compacted, when the audit log and TUI
companion remain in the right column, then the first viewport still contains
the review proof surfaces without document-level horizontal overflow.

## Conformance Cases

C-01 maps to AC-1: The proposal detail row markers remain unchanged.

C-02 maps to AC-2: The proposal detail exposes a vertical no-clipping budget
and a scoped max-height adjustment for its seven-row summary.

C-03 maps to AC-3: Browser geometry confirms the proposal detail section has
no hidden vertical overflow and the document has zero horizontal overflow.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3791`
- Browser geometry check against `/tmp/tau-harness-after.html` at 1512x1038.
