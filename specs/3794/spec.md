# Issue 3794: Harness Proposal Diff Route Is Operator-Readable

Status: Accepted
Priority: P2
Milestone: Tau Agent Harness UI hardening

## Problem

The live `View Diff` action on `/ops/harness` navigates to
`/ops/harness/proposals/PR-044/diff`, but that route returns an unstyled black
page with only a back link and a small pre block. It technically returns HTTP
200, but it is not an operator-usable diff review surface.

## Scope

In scope:
- Render the proposal diff route as a styled, standalone operator page.
- Preserve the proposal id, target path, dry-run status, and policy context.
- Keep the back link to the mission harness.

Out of scope:
- Changing proposal approval/apply semantics.
- Loading real patch files from disk.
- Reworking the full harness dashboard layout.

## Acceptance Criteria

AC-1: Given the live proposal diff route is opened, when the page renders, then
it exposes a styled diff review root with the selected proposal id.

AC-2: Given an operator reviews the diff, when they inspect the page, then they
can see target path, dry-run result, safety result, and policy scope.

AC-3: Given Browser Use opens the live diff route, when the page is inspected,
then console errors are zero and the diff content is visible without the bare
unstyled black-page failure.

## Conformance Cases

C-01 maps to AC-1: Gateway route tests assert `data-diff-view="operator-review"`
and `data-proposal-id="PR-044"`.

C-02 maps to AC-2: Gateway route tests assert visible target path, dry-run,
safety, allowed scope, and blocked scope markers.

C-03 maps to AC-3: Browser Use opens the localhost diff route and verifies the
styled diff root is visible.

## Success Signals

- `cargo test -p tau-gateway integration_spec_3757_c02_ops_harness_actions_are_stateful_and_gated`
- Browser Use screenshot of the live diff route.
