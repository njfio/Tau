# Issue 3790: Harness Tool Evidence Shows Full Memory Tool Names

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The mission proof table clips canonical memory tool names. `memory.search` and
`memory.write` overflow the Tool column and render as truncated abbreviations,
which weakens the proof view because operators cannot scan exact tool calls.

## Scope

In scope:
- Keep the compact proof evidence table in the first viewport.
- Keep Call ID hidden at the compact desktop layout.
- Show full canonical Tool names, including `memory.search` and
  `memory.write`.
- Preserve artifact path visibility and zero document-level horizontal
  overflow.

Out of scope:
- Changing proof row data or tool-call semantics.
- Reintroducing the hidden Call ID column.
- Reworking the proof section layout.

## Acceptance Criteria

AC-1: Given the `/ops/harness` proof table renders, when memory tool rows are
displayed, then `memory.search` and `memory.write` fit the Tool column without
horizontal clipping.

AC-2: Given compact proof evidence is active, when column widths are applied,
then the Tool column has a no-overflow budget and the Call ID column remains
hidden.

AC-3: Given the static preview is measured at the desktop harness viewport,
then the tool evidence table reports no visible cell overflow and no
document-level horizontal overflow.

## Conformance Cases

C-01 maps to AC-1: Memory tool rows remain present with full labels.

C-02 maps to AC-2: Tool evidence exposes a full-tool-label fit contract and
scoped column widths for Tool, Plan Node, Runtime, and Status.

C-03 maps to AC-3: Browser geometry confirms every visible Tool Evidence cell
fits within its client width.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3790`
- Browser geometry check against `/tmp/tau-harness-after.html` at 1512x1038.
