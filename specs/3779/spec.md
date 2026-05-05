Status: Accepted

# Spec 3779: Keep Recent Audit Proof Visible

## Problem

The self-improvement review pane now keeps actions, policy, and proposal safety
summary visible, but the recent audit log is still mostly clipped under the
review window. Operators need at least the latest audit proof rows visible in
the same first-viewport decision surface.

## Scope

In scope:
- Preserve the review order: learning queue, operator actions, conservative
  policy, proposal detail, then audit history.
- Add compact audit-history markers and a narrow audit table presentation for
  time, action, item, and result.
- Reduce only review-pane density enough for recent audit proof to fit without
  pushing the TUI companion out of view.

Out of scope:
- Changing audit storage or gateway state semantics.
- Removing full audit row data from the DOM.
- Changing proposal approval, apply, or safety policy mechanics.

## Acceptance Criteria

AC-1: Given `/ops/harness` renders, when the self-improvement review pane is
inspected, then the audit log remains after proposal detail and carries
first-viewport recent-proof markers.

AC-2: Given audit history is compacted, when rows render, then the visible audit
column priority is time, action, item, and result, while full row data remains in
the DOM.

AC-3: Given the right-side review pane is compacted, when the page is inspected
at desktop preview width, then audit history has a compact scroll region and the
TUI companion remains visible below the review pane.

## Conformance Cases

C-01 maps to AC-1. Render `/ops/harness` and assert proposal detail appears
before audit log, and the audit log carries first-viewport priority markers.

C-02 maps to AC-2. Render `/ops/harness` and assert audit rows preserve full
metadata while audit table CSS prioritizes time, action, item, and result.

C-03 maps to AC-3. Render `/ops/harness` and assert review density CSS,
compact audit max-height, and audit scroll-region markers are present.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3779` passes.
- Existing first-viewport harness tests continue to pass.
- Browser geometry confirms recent audit proof is inside the visible review
  pane and the TUI companion remains inside the viewport.
