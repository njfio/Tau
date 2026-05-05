# Plan 3779: Keep Recent Audit Proof Visible

## Approach

Add a focused conformance test for first-viewport audit visibility. Mark the
self-improvement review pane as compact audit priority, then add scoped CSS for
review density and a compact audit table. Keep all audit cells in the DOM while
using CSS to prioritize the columns operators need for recent proof scanning.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Compact audit styling could hide context operators need.
  Mitigation: Keep the full row data in DOM and make the audit table scrollable.
- Risk: Review density changes could disturb previous first-viewport contracts.
  Mitigation: Scope changes to the self-improvement pane and rerun the 3770-3779
  harness tests plus browser geometry.

## Interfaces

No route, endpoint, schema, audit storage, approval, apply, or safety semantics
change. This is a dashboard layout and marker contract only.
