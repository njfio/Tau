# Spec: Issue #3406 - Close GitHub milestone M291 after conformance completion

Status: Implemented

## Problem Statement
All M291 issues are closed and milestone docs report completed conformance, but the GitHub milestone `M291 - Tau E2E PRD Execution` remains in `open` state.

## Scope
In scope:
- Set GitHub milestone `291` state to `closed`.
- Record closure metadata in `specs/milestones/m291/index.md`.

Out of scope:
- Any runtime/test behavior changes.
- Any scenario remapping changes.

## Acceptance Criteria
### AC-1 GitHub milestone is formally closed
Given no open issues remain in milestone 291,
when milestone metadata is queried,
then milestone state is `closed`.

### AC-2 Repository milestone index records closure metadata
Given milestone close action is completed,
when milestone index is reviewed,
then closeout section includes closure record with issue/PR trace.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Process/Conformance | milestone 291 has open_issues=0 | GitHub milestone state updated | API returns `state=closed` |
| C-02 | AC-2 | Docs/Conformance | m291 index exists | closure note added | index includes milestone closure metadata |

## Success Metrics / Observable Signals
- `gh api repos/njfio/Tau/milestones/291` reports `state: closed`.
- `specs/milestones/m291/index.md` includes a closure note with reference to issue `#3406`.
