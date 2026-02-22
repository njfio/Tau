# Spec: Issue #3304 - Sync M251 milestone index status with closed milestone state

Status: Implemented

## Problem Statement
GitHub milestone `M251` is closed with all linked issues completed, but `specs/milestones/m251/index.md` still reports `Status: In Progress`, causing repository lifecycle metadata drift.

## Scope
In scope:
- Update `specs/milestones/m251/index.md` status and closeout note to reflect completed state.
- Keep milestone-linked issue references and merged completion references consistent.
- Add per-issue lifecycle artifacts for #3304 (`spec.md`, `plan.md`, `tasks.md`).

Out of scope:
- Runtime/algorithm changes.
- New API/behavior changes.

## Acceptance Criteria
### AC-1 Milestone index reflects closed state
Given GitHub milestone `M251` is closed with no open issues,
when `specs/milestones/m251/index.md` is read,
then it reports completed status and includes concise closeout context referencing delivered issues/PR.

### AC-2 Lifecycle artifacts are complete
Given issue `#3304`,
when repository specs are inspected,
then `specs/3304/spec.md`, `specs/3304/plan.md`, and `specs/3304/tasks.md` exist and are marked implemented/completed at closeout.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Conformance/Docs | milestone index file | check status line/closeout section | status is completed and closeout references merged completion work |
| C-02 | AC-2 | Conformance/Process | issue lifecycle artifact paths | check files/status entries | spec/plan/tasks exist and are completed |

## Success Metrics / Observable Signals
- `rg -n "^Status: Completed" specs/milestones/m251/index.md`
- `rg -n "Closeout|#3303|#3302|#3300|#3296" specs/milestones/m251/index.md`
- `test -f specs/3304/spec.md && test -f specs/3304/plan.md && test -f specs/3304/tasks.md`
