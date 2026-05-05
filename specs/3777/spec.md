Status: Accepted

# Spec 3777: Keep Conservative Review Policy Visible

## Problem

The self-improvement review pane can hide the conservative change policy below
the visible review area. That policy is the safety boundary for the apply flow,
so it should be visible before the longer proposal explanation.

## Scope

In scope:
- Promote Conservative Change Policy above the long proposal detail body.
- Preserve operator actions as the first actionable controls after the learning
  queue.
- Preserve the existing approval-gated apply semantics, allowed targets, blocked
  targets, and audit history.

Out of scope:
- Changing self-improvement apply mechanics.
- Making source code or safety policy autonomous apply targets.
- Removing proposal detail or audit history.

## Acceptance Criteria

AC-1: Given the self-improvement review window renders, when the learning queue
and operator actions are present, then Conservative Change Policy appears after
operator actions and before the proposal detail.

AC-2: Given the apply flow remains conservative, when the policy is promoted,
then allowed targets remain skill, config, and prompt, while blocked targets
remain source code and safety policy.

AC-3: Given the audit log remains part of review history, when the policy and
proposal detail are reordered, then audit history still renders after proposal
detail.

## Conformance Cases

C-01 maps to AC-1. Render `/ops/harness` and assert the review order is
learning queue -> operator actions -> conservative policy -> proposal detail.

C-02 maps to AC-2. Render `/ops/harness` and assert the conservative policy
section carries the first-viewport priority marker plus the existing allowed and
blocked target metadata.

C-03 maps to AC-3. Render `/ops/harness` and assert proposal detail remains
before audit history, and audit history still renders.

## Success Signals

- `cargo test -p tau-dashboard-ui functional_spec_3777` passes.
- Existing self-improvement action-priority tests continue to pass.
- Browser geometry confirms Conservative Change Policy is inside the visible
  self-improvement review window.
