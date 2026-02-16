# Tasks #2029

Status: Implemented
Spec: specs/2029/spec.md
Plan: specs/2029/plan.md

## Ordered Tasks
- T1 (RED): carry forward red evidence from child story/task implementation
  phases where lifecycle artifacts and deliverables were initially missing.
- T2 (GREEN): map completed child stories/tasks to epic AC-1..AC-3 with
  concrete conformance criteria.
- T3 (VERIFY): run live governance checks:
  `gh issue list --search "repo:njfio/Tau \"Parent: #2029\"" --state all ...`,
  `gh issue list --milestone "M25 Governance + Decomposition + Velocity" --state open ...`,
  `scripts/dev/test-roadmap-status-sync.sh`,
  `python3 .github/scripts/test_roadmap_status_workflow_contract.py`,
  `scripts/dev/roadmap-status-sync.sh --check --quiet`,
  plus child-task lifecycle status checks for `#2045/#2046/#2047/#2048`.
- T4 (CLOSE): mark `specs/2029/*` implemented, merge closure PR, and close epic
  `#2029` with `status:done`.
