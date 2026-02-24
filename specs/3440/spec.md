# Spec: Issue #3440 - Align planning docs with delivered integration status

Status: Implemented

## Problem Statement
Two planning docs currently describe true RL/auth/dashboard/TUI as future-gap areas using stale language, which conflicts with implemented conformance evidence and current README/operator guidance.

## Scope
In scope:
- Update:
  - `docs/planning/integration-gap-closure-plan.md`
  - `docs/planning/true-rl-roadmap-skeleton.md`
- Distinguish clearly:
  - implemented/integrated status today,
  - remaining expansion or long-horizon work.
- Keep roadmap structure and links intact where still relevant.

Out of scope:
- Runtime code changes.
- New roadmap stage design.
- CI workflow modifications.

## Acceptance Criteria
### AC-1 Integration gap plan reflects current delivered baseline
Given `docs/planning/integration-gap-closure-plan.md`,
when reviewed after the change,
then it no longer frames true RL/auth/dashboard/TUI as unresolved baseline gaps where deterministic implementation and conformance already exist.

### AC-2 True RL roadmap skeleton states current boundary accurately
Given `docs/planning/true-rl-roadmap-skeleton.md`,
when reviewed after the change,
then it explicitly marks current integrated true-RL baseline and positions listed stages as expansion hardening roadmap rather than first-time delivery.

### AC-3 Docs quality gates remain green
Given docs updates,
when docs and roadmap sync checks run,
then they pass without regressions.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | integration-gap plan doc | inspect updated sections | status wording aligns with implemented baseline |
| C-02 | AC-2 | Functional | true-RL roadmap skeleton | inspect current-state boundary + stage framing | roadmap describes expansion/hardening path |
| C-03 | AC-3 | Regression | docs quality scripts | run docs checks + roadmap sync check | all commands exit zero |

## Success Metrics / Observable Signals
- Planning docs and README no longer send conflicting signals about delivery state.
- Operators/readers can identify what is delivered now vs what is expansion work.

## Implementation Evidence
### RED
- `rg -n "planned/future|not yet|staged primitives|gap" docs/planning/integration-gap-closure-plan.md docs/planning/true-rl-roadmap-skeleton.md`
  - captured stale wording before patch (future-only RL boundary and gap-only framing).

### GREEN
- `python3 .github/scripts/architecture_docs_check.py --repo-root .`
- `python3 .github/scripts/runbook_ownership_docs_check.py --repo-root .`
- `scripts/dev/test-roadmap-status-sync.sh`
- `scripts/dev/roadmap-status-sync.sh --check --quiet`
