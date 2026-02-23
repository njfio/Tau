# Spec: Issue #3404 - Finalize M291 milestone closeout status

Status: Implemented

## Problem Statement
`specs/milestones/m291/index.md` still reflects in-progress milestone state after final delivery (`#3402` / `#3403`) and does not explicitly capture the terminal conformance signal (`Covered=112`, `N/A=0`).

## Scope
In scope:
- Update milestone status and closeout phase language to reflect full delivery.
- Record final conformance end-state in milestone success signals.

Out of scope:
- Runtime behavior, test logic, or conformance-row remapping changes.

## Acceptance Criteria
### AC-1 milestone status reflects completion
Given M291 linked tasks are merged and parent story/epic are closed,
when the milestone index is reviewed,
then `Status` reflects completion rather than in-progress state.

### AC-2 closeout phase progression reflects final delivered phase
Given phase log entries in M291 index,
when reviewed,
then Phase 9 reflects delivered (not in progress) state.

### AC-3 success signals include final conformance state
Given the conformance matrix reports all rows covered,
when milestone success signals are reviewed,
then final coverage summary (`Covered=112`, `N/A=0`) is explicitly recorded.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Docs/Conformance | M291 index header | status updated | reads completed state |
| C-02 | AC-2 | Docs/Conformance | M291 closeout section | phase line updated | phase 9 marked delivered |
| C-03 | AC-3 | Docs/Conformance | M291 success signals | final coverage signal added | includes `Covered=112`, `N/A=0` |

## Success Metrics / Observable Signals
- `specs/milestones/m291/index.md` reflects completed milestone/phase.
- Coverage summary line in success signals matches matrix (`Covered=112`, `N/A=0`).
