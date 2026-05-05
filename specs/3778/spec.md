Status: Accepted

# Spec 3778: Keep Proposal Safety Summary Visible

## Problem

The self-improvement review pane now keeps operator actions and Conservative
Change Policy visible, but the proposal detail section can still clip under the
review window. Operators need the dry-run result, safety check, and rollback
plan visible near the policy before deciding whether to approve a proposal.

## Scope

In scope:
- Keep proposal detail immediately after Conservative Change Policy.
- Prioritize dry-run result, safety check, and rollback plan at the top of the
  proposal detail section.
- Constrain proposal detail to a compact scroll area so it remains inside the
  first review viewport.
- Preserve all existing proposal rows, links, approval controls, policy
  metadata, and audit history.

Out of scope:
- Changing approval-gated apply mechanics.
- Changing proposal storage, routing, or audit semantics.
- Removing failure/root-cause context from proposal detail.

## Acceptance Criteria

AC-1: Given the self-improvement review window renders, when Conservative
Change Policy appears, then proposal detail appears immediately after it with a
first-viewport compact-scroll marker.

AC-2: Given an operator evaluates PR-044, when proposal detail renders, then
Dry-run Result, Safety Check, and Rollback Plan appear before lower-priority
patch/failure/root-cause explanation.

AC-3: Given proposal detail is compacted, when audit history renders, then all
proposal detail rows still exist and audit history remains after proposal
detail.

## Conformance Cases

C-01 maps to AC-1. Render `/ops/harness` and assert Conservative Change Policy
appears before proposal detail, and proposal detail carries first-viewport
compact-scroll markers.

C-02 maps to AC-2. Render `/ops/harness` and assert Dry-run Result, Safety
Check, and Rollback Plan are the first proposal detail rows before Patch
Summary, Failure Observed, and Root Cause.

C-03 maps to AC-3. Render `/ops/harness` and assert proposal detail keeps test
evidence plus failure/root-cause rows, and audit history renders after proposal
detail.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3778` passes.
- Existing first-viewport harness tests continue to pass.
- Browser geometry confirms proposal detail stays within the visible
  self-improvement review window without pushing the TUI companion out of the
  viewport.
