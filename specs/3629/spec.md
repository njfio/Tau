# Spec: Issue #3629 - Close live_rl_runtime oversized-file blocker

Status: Reviewed
Priority: P1
Milestone: M330

## Problem Statement
The shared CI quality lane now fails on the oversized-file guard because
`crates/tau-coding-agent/src/live_rl_runtime.rs` is 4460 lines, above the
default 4000-line threshold. The failure exists on the current base branch, so
it is blocking unrelated review work as well as the eventual decomposition fix.

This epic creates the governed execution path for closing that blocker without
burying the work inside unrelated PRs or bypassing the oversized-file policy.

## Scope
In scope:
- establish a live milestone container for the blocker;
- define a concrete story for decomposing `live_rl_runtime.rs`;
- capture the binding spec/plan/tasks artifacts for that story;
- track the implementation and verification needed to restore the quality lane.

Out of scope:
- mixing the decomposition into PR #3628;
- adding broad oversized-file exemptions as a substitute for decomposition;
- unrelated RL feature work or behavior changes.

## Acceptance Criteria
### AC-1 Live planning hierarchy exists for the blocker
Given the old oversized-file decomposition wave is closed,
when maintainers inspect the repo and GitHub hierarchy,
then a new active milestone and issue chain exist for the
`live_rl_runtime.rs` blocker.

### AC-2 A concrete implementation story is specified
Given the blocker requires runtime refactoring,
when engineers pick up the work,
then they have a reviewed story spec/plan/tasks bundle that defines
scope, acceptance criteria, conformance cases, and verification commands.

### AC-3 Epic closure requires restored policy compliance
Given the story implementation is complete,
when this epic is closed,
then the oversized-file guard is green for `live_rl_runtime.rs`
without a new exemption and targeted runtime contracts remain verified.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | repo governance artifacts | inspect milestone and linked issues | active `M330` + `#3629/#3630` hierarchy exists |
| C-02 | AC-2 | Conformance | story issue `#3630` | inspect `specs/3630/*` | reviewed spec/plan/tasks artifacts exist and match the story scope |
| C-03 | AC-3 | Functional | completed implementation branch | run oversized-file guard and targeted tests | policy passes without new exemption and runtime selectors remain green |

## Success Metrics / Observable Signals
- `M330` exists in GitHub and `specs/milestones/m330/index.md` exists in git.
- Story `#3630` has a reviewed spec bundle checked into the repo.
- Epic closure happens only after the story restores oversized-file guard
  compliance for `live_rl_runtime.rs`.
