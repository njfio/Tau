# Tasks: Issue #3424 - Full-delivery program plan

## Planning Artifact Tasks (this issue)
- [x] T1 (RED, Conformance): verify required artifact paths for `M296/#3424` are missing before authoring.
- [x] T2 (GREEN, Docs): create `specs/milestones/m296/index.md` with milestone context/scope/exit criteria.
- [x] T3 (GREEN, Spec): create `specs/3424/spec.md` with AC-1..AC-6 and conformance cases.
- [x] T4 (GREEN, Plan): create `specs/3424/plan.md` with phased execution, gates, risks, and decomposition strategy.
- [x] T5 (VERIFY, Conformance): run artifact and consistency checks for milestone/spec/plan/tasks package.

## Program Execution Streams (child tasks under M296)
- [x] T6 (RED -> GREEN, Functional/Conformance): True RL E2E productionization stream (`#3427`).
- [x] T7 (RED -> GREEN, Integration/Functional): Dashboard and TUI convergence stream (`#3428`).
- [x] T8 (RED -> GREEN, Functional/Regression): Auth lifecycle verification/hardening stream (`#3426`).
- [x] T9 (RED -> GREEN, Integration/Regression): Cross-stream integrated reliability and recovery stream (`#3429`).
- [x] T10 (VERIFY, Conformance): GA readiness and rollback validation stream (`#3430`).

## Verification Evidence (2026-02-23)

### RED
- `ls -la specs/3424`
- `ls -la specs/milestones/m296`
- Result: both paths were missing in this worktree prior to artifact creation.

### GREEN
- Added:
  - `specs/milestones/m296/index.md`
  - `specs/3424/spec.md`
  - `specs/3424/plan.md`
  - `specs/3424/tasks.md`

### REGRESSION
- `rg --files specs/3424`
- `rg -n "Status: Implemented|## Acceptance Criteria|## Conformance Cases" specs/3424/spec.md`
- `rg -n "## Phase Plan|## Risks and Mitigations|## Interfaces / Contracts" specs/3424/plan.md`
- `rg -n "Planning Artifact Tasks|Program Execution Streams|Verification Evidence" specs/3424/tasks.md`

### CLOSEOUT (2026-02-23)
- `gh issue view 3426 --json state`
- `gh issue view 3427 --json state`
- `gh issue view 3428 --json state`
- `gh issue view 3429 --json state`
- `gh issue view 3430 --json state`
- `gh api repos/njfio/Tau/milestones/296`
- Result: all phase issues are `CLOSED`; milestone `296` is `closed` with `open_issues: 0`.

## Test Tier Matrix (for planning artifact issue)
| Tier | ✅/❌/N/A | Tests / Evidence | N/A Why |
|---|---|---|---|
| Unit | N/A | Docs-only issue | No runtime code changed |
| Property | N/A | Docs-only issue | No invariant-bearing code changed |
| Contract/DbC | N/A | Docs-only issue | No DbC annotations/interfaces changed |
| Snapshot | N/A | Docs-only issue | No snapshot artifacts changed |
| Functional | ✅ | AC/conformance structure validated in spec/plan/tasks | |
| Conformance | ✅ | artifact presence + section checks via `rg` | |
| Integration | N/A | Docs-only issue | No cross-module runtime behavior changed |
| Fuzz | N/A | Docs-only issue | No untrusted-input runtime path changed |
| Mutation | N/A | Docs-only issue | No production code diff to mutate |
| Regression | ✅ | structure checks after write | |
| Performance | N/A | Docs-only issue | No performance-sensitive path changed |
