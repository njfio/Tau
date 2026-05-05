Status: Accepted

# Spec 3776: Keep Proof Memory And Artifacts Visible

## Problem

The mission detail proof window still pushes Memory / Learning and Artifacts
below the first viewport. Those panels are part of the mission proof trail, so
the operator should not need to scroll the proof window just to see whether
memory writes and artifact outputs exist.

## Scope

In scope:
- Keep Memory / Learning and Artifacts visible in the first desktop proof
  viewport.
- Preserve the existing proof ordering: tool evidence, operator log,
  acceptance, verification gates, memory, artifacts.
- Compact only the secondary acceptance and gate detail panels with local scroll
  budgets.

Out of scope:
- Changing verification gate semantics.
- Removing acceptance criteria or gate items.
- Changing proof artifact links or memory counters.

## Acceptance Criteria

AC-1: Given the mission detail proof window renders on the desktop harness
layout, when the acceptance and verification-gate panels contain five items,
then those secondary panels use a compact local scroll budget.

AC-2: Given Memory / Learning and Artifacts are proof outputs, when the proof
window renders, then both sections are marked as first-viewport proof footer
panels.

AC-3: Given proof review depends on stable ordering, when the layout is
compacted, then operator log, acceptance, gates, memory, and artifacts retain
their existing order.

## Conformance Cases

C-01 maps to AC-1. Render `/ops/harness` and assert Acceptance Criteria and
Verification Gates expose compact proof-detail budget markers and matching CSS.

C-02 maps to AC-2. Render `/ops/harness` and assert Memory / Learning and
Artifacts expose first-viewport proof-footer markers.

C-03 maps to AC-3. Render `/ops/harness` and assert the proof sections retain
operator-log -> acceptance -> gates -> memory -> artifacts ordering.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3776` passes.
- Existing proof prioritization tests continue to pass.
- Browser geometry confirms Memory / Learning and Artifacts are inside the
  first viewport.
