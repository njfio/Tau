# Spec: Issue #3312 - Produce Review #40 repository quality/status report

Status: Implemented

## Problem Statement
After merging Review #39 delivery work, the repository needs a fresh snapshot (`Review #40`) on current `origin/master` to quantify immediate post-merge deltas and reconfirm current quality/risk posture.

## Scope
In scope:
- Produce `tasks/review-40.md` with updated repository metrics and deltas versus Review #39.
- Summarize merged changes since Review #39 baseline commit.
- Record current self-improvement loop status and remaining-risk assessment.

Out of scope:
- Runtime feature implementation.
- New architecture changes.

## Acceptance Criteria
### AC-1 Review #40 artifact exists with current baseline
Given current `origin/master`,
when `tasks/review-40.md` is read,
then it includes date, baseline commit hash, and updated repository metrics.

### AC-2 Review captures delta since Review #39
Given Review #39 baseline commit `75fb23d9`,
when the report is inspected,
then it includes commit/PR delta summary and updated verdict.

### AC-3 Lifecycle artifacts exist and are implemented
Given issue `#3312`,
when specs are inspected,
then `specs/3312/spec.md`, `specs/3312/plan.md`, and `specs/3312/tasks.md` exist and are complete.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Conformance/Docs | current `origin/master` | read review file header/metrics | baseline commit + updated metrics present |
| C-02 | AC-2 | Conformance/Docs | commit range `75fb23d9..origin/master` | read change summary section | merged PR list + verdict present |
| C-03 | AC-3 | Conformance/Process | issue artifact paths | validate files/status markers | spec/plan/tasks exist and are implemented |

## Success Metrics / Observable Signals
- `test -f tasks/review-40.md`
- `rg -n "origin/master HEAD|Review #40|Overall" tasks/review-40.md`
- `test -f specs/3312/spec.md && test -f specs/3312/plan.md && test -f specs/3312/tasks.md`
