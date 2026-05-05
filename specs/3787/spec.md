# Spec 3787: Proposal Patch Summary Readability

Status: Implemented

## Problem

The Self-Improvement Review proposal detail shows the selected proposal, but the Patch Summary value is clipped with an ellipsis. Operators need the complete proposed change summary visible before deciding whether to approve, reject, dry-run, or inspect the diff.

## Acceptance Criteria

AC-1: Given `/ops/harness` is rendered in the desktop harness preview, when the proposal detail is visible, then the Patch Summary value is readable without horizontal clipping.

AC-2: Given the proposal detail remains compact, when the Patch Summary is made readable, then all seven proposal proof rows remain present before the audit log.

AC-3: Given the right review pane is rendered, when the proposal detail expands the Patch Summary row, then the audit log and TUI companion remain inside the first viewport with no horizontal document overflow.

## Scope

In scope:
- Tau Agent Harness proposal detail HTML/CSS contract.
- Patch Summary row readability.
- Conformance test plus browser geometry validation.

Out of scope:
- Changing proposal data, operator action behavior, policy cards, audit data, or gateway behavior.
- TUI behavior.

## Conformance Cases

C-01 maps AC-1: Render `/ops/harness` and assert Patch Summary declares full-text fit with no summary overflow budget.

C-02 maps AC-2: Render `/ops/harness` and assert all seven proposal detail rows remain present before the audit log.

C-03 maps AC-3: Browser geometry checks Patch Summary truncation is zero and the review/TUI surfaces remain in viewport.

## Success Metrics

- `cargo test -p tau-dashboard-ui functional_spec_3787` passes.
- Browser geometry confirms Patch Summary overflow is zero, no document horizontal overflow, and the right review/TUI surfaces remain in viewport.
- Existing harness regression tests remain green.
