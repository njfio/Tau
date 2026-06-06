# Spec: Issue #3766 - Refresh roadmap status snapshot after proof-loop landing

Status: Accepted
Priority: P2
Milestone: M334 - Tau Ralph loop supervisor architecture
Parent: #3654

## Problem Statement

After the Agent Canvas proof-loop slice landed, the generated roadmap status
docs drifted from the current status snapshot date. The repository check
`scripts/dev/roadmap-status-sync.sh --check --quiet` now fails until the
generated sections are refreshed from updated `master`.

## Scope

In scope:

- Regenerate roadmap status sections with `scripts/dev/roadmap-status-sync.sh`.
- Keep changes limited to generated roadmap status docs and this issue's spec
  artifacts.
- Verify the roadmap status sync check and focused Agent Canvas proof regression
  pass after the refresh.

Out of scope:

- Editing roadmap content by hand.
- Changing Agent Canvas proof-loop behavior.
- Reopening or changing issue #3764.

## Acceptance Criteria

AC-1: Given updated `master`, when `scripts/dev/roadmap-status-sync.sh --check
--quiet` runs, then it exits successfully.

AC-2: Given the generated roadmap docs are refreshed, when the diff is reviewed,
then it contains only the current snapshot refresh and this issue's process
artifacts.

AC-3: Given the proof-loop slice is already landed, when the focused proof
regression runs, then `scripts/dev/test-ops-chat-canvas-proof.sh` still passes.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | refreshed generated docs | run roadmap sync check | command exits 0 |
| C-02 | AC-2 | Regression | generated refresh diff | inspect changed files | only generated status and spec artifacts changed |
| C-03 | AC-3 | Functional | current master with proof-loop merge | run focused shell proof test | proof-loop regression passes |

## Success Metrics / Observable Signals

- `scripts/dev/roadmap-status-sync.sh --check --quiet`
- `scripts/dev/test-ops-chat-canvas-proof.sh`
- `git diff --check`
