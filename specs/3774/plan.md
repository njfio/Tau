# Plan 3774: Keep Operator Review Actions First-Viewport Visible

## Approach

Move the operator action section above the proposal detail in the
self-improvement review window. This keeps the high-priority approval controls
visible before the longer explanatory proposal body. Add a dedicated placement
marker so static tests can distinguish the stronger contract from the earlier
"actions before policy" contract.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Reordering content could weaken proposal context before approval.
  Mitigation: Keep the learning queue immediately above the actions and leave
  the detail content directly below the actions in the same review window.
- Risk: Existing tests may still expect the weaker placement marker.
  Mitigation: Update the existing self-improvement priority test to accept the
  stronger placement marker while preserving endpoint and policy checks.

## Interfaces

No route, endpoint, schema, or approval semantics change. This is a dashboard
markup and visual-priority change only.
