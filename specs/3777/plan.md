# Plan 3777: Keep Conservative Review Policy Visible

## Approach

Move Conservative Change Policy directly after Operator Actions and before the
long proposal detail section. Add a deterministic marker for first-viewport
policy priority. This keeps the safety boundary visible near the approval
controls without changing endpoints, apply semantics, or audit history.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Proposal detail context moves below the policy.
  Mitigation: Keep proposal detail immediately after policy in the same review
  pane, and keep all detail rows intact.
- Risk: Existing action-priority tests become stale.
  Mitigation: Preserve the action-before-detail and action-before-policy
  contracts.

## Interfaces

No route, endpoint, schema, approval, audit, or self-improvement semantics
change. This is a review-pane layout and marker contract only.
