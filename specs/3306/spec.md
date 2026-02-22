# Spec: Issue #3306 - Mark m252 milestone index completed

Status: Implemented

## Problem Statement
After merging #3305, milestone `M252` achieved its documented objective, but `specs/milestones/m252/index.md` still reports `Status: In Progress`, leaving lifecycle metadata inconsistent.

## Scope
In scope:
- Update `specs/milestones/m252/index.md` to completed status.
- Add a closeout note referencing merged completion work (#3305, #3304).
- Create per-issue lifecycle artifacts (`spec.md`, `plan.md`, `tasks.md`) for #3306.

Out of scope:
- Any runtime, API, or behavior changes.

## Acceptance Criteria
### AC-1 M252 index is completed
Given milestone objective work is merged,
when `specs/milestones/m252/index.md` is read,
then status is completed and closeout note references #3305 and #3304.

### AC-2 Issue lifecycle artifacts are complete
Given issue #3306,
when repository specs are inspected,
then `specs/3306/spec.md`, `specs/3306/plan.md`, and `specs/3306/tasks.md` exist and are implemented.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Conformance/Docs | m252 milestone index file | validate status + closeout references | status is completed and closeout references #3305/#3304 |
| C-02 | AC-2 | Conformance/Process | lifecycle artifact paths | validate files/status | spec/plan/tasks exist and are completed |

## Success Metrics / Observable Signals
- `rg -n "^Status: Completed" specs/milestones/m252/index.md`
- `rg -n "Closeout|#3305|#3304" specs/milestones/m252/index.md`
- `test -f specs/3306/spec.md && test -f specs/3306/plan.md && test -f specs/3306/tasks.md`
