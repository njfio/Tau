Status: Accepted

# Spec 3780: Keep All Verification Gates Visible

## Problem

The proof pane keeps the verification-gate card in the first viewport, but the
fifth gate chip can be clipped inside the card. Operators need all five gate
states visible together because the gate set is the proof checklist for the
mission.

## Scope

In scope:
- Preserve the existing proof order: operator log, acceptance, verification
  gates, memory, artifacts.
- Keep all five verification gate rows in the DOM and visible in the card.
- Add a compact all-gates-first-viewport layout marker for the verification
  gate list.

Out of scope:
- Changing gate semantics, status values, or proof source data.
- Changing memory/artifact placement.
- Changing benchmark or self-improvement review layout.

## Acceptance Criteria

AC-1: Given `/ops/harness` renders, when the proof pane is inspected, then the
verification-gate section carries an all-gates-first-viewport marker.

AC-2: Given verification gates render, when the gate list is inspected, then
VG-01 through VG-05 are present in order and the layout uses compact two-column
gate density rather than a clipped vertical list.

AC-3: Given the proof footer remains visible, when gate density changes, then
memory and artifacts still render after verification gates.

## Conformance Cases

C-01 maps to AC-1. Render `/ops/harness` and assert the verification-gate
section has all-gates-first-viewport metadata.

C-02 maps to AC-2. Render `/ops/harness` and assert all five gate IDs appear in
order and compact gate CSS is present.

C-03 maps to AC-3. Render `/ops/harness` and assert memory/artifact proof footer
sections remain after verification gates.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3780` passes.
- Existing first-viewport harness tests continue to pass.
- Browser geometry confirms all verification gate chips fit inside the gate
  card and the proof footer remains visible.
