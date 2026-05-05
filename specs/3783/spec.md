# Spec 3783: Right Review Pane Contained Proof Rows

Status: Reviewed

## Problem

The Tau Agent Harness right review pane still hides or clips important proof content inside compact sections. The learning queue hides its fourth item, proposal detail hides the failure/root-cause/evidence rows, and the audit proof clips the rejected row at the bottom of the card.

## Acceptance Criteria

AC-1: Given `/ops/harness` is rendered, when the Learning & Proposals section is shown, then all four queue entries are visible without section-internal clipping.

AC-2: Given `/ops/harness` is rendered, when the selected proposal detail is shown, then all seven proposal proof rows are contained within the proposal card without visually extending into the audit card.

AC-3: Given `/ops/harness` is rendered, when the recent audit log is shown, then all four audit proof rows fit inside the audit card.

## Scope

In scope:
- Tau Agent Harness right review pane CSS/HTML contracts.
- Learning queue, proposal detail, and audit log compact density.
- Conformance tests and browser geometry validation.

Out of scope:
- Changing self-improvement approval semantics.
- Backend audit/proposal state changes.
- TUI command behavior.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert the learning queue declares all-items-visible density while retaining all four entries.

C-02 maps AC-2: Render `/ops/harness` and assert proposal detail declares a contained seven-row proof budget with compact row/link styling.

C-03 maps AC-3: Render `/ops/harness` and assert audit log declares all-rows-visible density while retaining the four fallback audit rows.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3783` passes.
- Browser geometry confirms zero child overflow in learning queue, proposal detail, and audit log sections.
- The TUI companion remains visible in the first viewport.
