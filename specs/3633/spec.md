# Spec: Issue #3633 - Repair broken relative links in March 24 planning docs

Status: Implemented

## Objective
Repair the two broken relative links in the March 24 planning documents so
`docs_link_check` no longer fails on those files.

## Inputs/Outputs
Inputs:
- Existing plan docs under `docs/plans/`.
- Current markdown link checker behavior from
  `.github/scripts/docs_link_check.py`.

Outputs:
- Correct relative links from the March 24 plan docs to their intended
  targets.
- Green docs-link verification for the touched docs.

## Boundaries / Non-goals
In scope:
- `docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md`
- `docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md`
- Spec artifacts for `#3633`

Out of scope:
- Broader content edits in the March 24 plans.
- CI workflow logic changes.
- Formatting or wording cleanup unrelated to broken targets.

## Failure Modes
- The first plan still points from `docs/plans/` to `docs/ideation/...`,
  which resolves to a non-existent nested path.
- The second plan still points from `docs/plans/` to
  `docs/plans/2026-03-24-001-...`, which also resolves to a non-existent
  nested path.
- A fix changes the prose but leaves one of the actual markdown links broken.

## Acceptance Criteria
### AC-1 First plan links correctly to the ideation origin
Given the March 24 radical simplification plan,
when markdown links are resolved relative to its file location,
then the origin link targets the real ideation document under `docs/ideation/`.

### AC-2 Second plan links correctly to the first plan
Given the March 24 endgame plan,
when markdown links are resolved relative to its file location,
then the origin link targets the real March 24 plan in the same directory.

### AC-3 Docs link verification is green for the repaired targets
Given the repository root,
when `python3 .github/scripts/docs_link_check.py --repo-root .` runs after the
fix,
then it reports no issues for the two March 24 planning docs.

## Conformance Cases
- C-01 / AC-1 / Functional:
  `docs/plans/2026-03-24-001-...` uses `../ideation/...` in markdown link
  positions that are intended to reach the ideation doc.
- C-02 / AC-2 / Functional:
  `docs/plans/2026-03-24-002-...` uses a same-directory relative target for
  the March 24 implementation plan.
- C-03 / AC-3 / Conformance:
  `docs_link_check.py` reports `issues=0` for the repository after the two
  link repairs.

## Files To Touch
- `specs/milestones/m330/index.md`
- `specs/3633/spec.md`
- `specs/3633/plan.md`
- `specs/3633/tasks.md`
- `docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md`
- `docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md`

## Test Plan
- Red: run `python3 .github/scripts/docs_link_check.py --repo-root .` and
  capture the two current failures.
- Green: rerun the same checker after the docs edits and verify the failures
  are gone.

## Success Metrics / Observable Signals
- The two broken-link findings disappear from docs-link output.
- No additional docs-link findings are introduced by this change.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md` now targets `../ideation/2026-03-24-radical-simplification-self-improvement-ideation.md` for its origin references. |
| AC-2 | ✅ | `docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md` now targets `2026-03-24-001-feat-radical-simplification-self-improvement-plan.md` from the same directory. |
| AC-3 | ✅ | `python3 .github/scripts/docs_link_check.py --repo-root .` reports `issues=0`. |

## Validation
- RED: `python3 .github/scripts/docs_link_check.py --repo-root .`
  - Before the fix: `issues=2`
  - Findings:
    - `docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md`
    - `docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md`
- GREEN: `python3 .github/scripts/docs_link_check.py --repo-root .`
  - After the fix: `issues=0`
