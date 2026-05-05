Status: Accepted

# Spec 3774: Keep Operator Review Actions First-Viewport Visible

## Problem

After the TUI companion was moved into the first viewport, the self-improvement
review window became shorter. The operator action controls can fall below the
visible review area, which makes the most important approval flow feel hidden.

## Scope

In scope:
- Prioritize all self-improvement operator action controls above long proposal
  detail content in the Tau Agent Harness dashboard.
- Preserve the conservative approval-gated policy and existing action endpoints.
- Add deterministic dashboard markup checks for the placement contract.

Out of scope:
- Adding new gateway integrations.
- Changing proposal approval semantics.
- Making autonomous apply bypass approval.

## Acceptance Criteria

AC-1: Given the Tau Agent Harness dashboard renders the self-improvement review
window, when the learning queue and proposal detail are present, then the
operator actions appear immediately after the queue and before the proposal
detail.

AC-2: Given the operator actions are first-viewport priority controls, when the
dashboard renders them, then approve, reject, dry-run, view-diff, and disabled
apply controls are all present in the prioritized action section.

AC-3: Given self-improvement changes remain conservative, when the operator
actions are prioritized, then the approval-gated metadata and conservative
policy content remain present.

## Conformance Cases

C-01 maps to AC-1, AC-3. Render `/ops/harness` and assert
`tau-ops-harness-learning-queue` appears before
`tau-ops-harness-operator-actions`, which appears before
`tau-ops-harness-proposal-detail` and conservative policy.

C-02 maps to AC-2. Render `/ops/harness` and assert every operator action form,
button, and link remains present in the action section.

C-03 maps to AC-3. Render `/ops/harness` and assert the review placement marker,
approval-required marker, and conservative policy marker remain present.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3774` passes.
- Existing self-improvement action priority tests continue to pass.
- Browser geometry check confirms all operator controls fit within the visible
  self-improvement review window.
