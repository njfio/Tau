# Plan #2029

Status: Implemented
Spec: specs/2029/spec.md

## Approach

1. Treat merged story/task closure PRs as implementation evidence and codify
   epic-level conformance mapping in `specs/2029/*`.
2. Run live issue/milestone status queries to verify child-story completion and
   milestone-open-issue state.
3. Run roadmap sync/contract checks to ensure no drift in governance source of
   truth.
4. Verify child task lifecycle artifact completeness for
   `#2045/#2046/#2047/#2048`.
5. Close epic with `status:done` and closure summary linked to final PR.

## Affected Modules

- `specs/2029/spec.md`
- `specs/2029/plan.md`
- `specs/2029/tasks.md`
- `scripts/dev/test-roadmap-status-sync.sh`
- `.github/scripts/test_roadmap_status_workflow_contract.py`
- `scripts/dev/roadmap-status-sync.sh`
- `specs/2045/spec.md`
- `specs/2045/plan.md`
- `specs/2045/tasks.md`
- `specs/2046/spec.md`
- `specs/2046/plan.md`
- `specs/2046/tasks.md`
- `specs/2047/spec.md`
- `specs/2047/plan.md`
- `specs/2047/tasks.md`
- `specs/2048/spec.md`
- `specs/2048/plan.md`
- `specs/2048/tasks.md`

## Risks and Mitigations

- Risk: issue status drift between local assumptions and live GitHub state.
  - Mitigation: use live `gh issue`/milestone queries during verification.
- Risk: epic closes while lifecycle artifacts are incomplete.
  - Mitigation: explicit conformance check for spec/plan/tasks implemented
    status in all child tasks.

## Interfaces and Contracts

- Live issue query:
  `gh issue list --search "repo:njfio/Tau \"Parent: #2029\"" --state all ...`
- Milestone open query:
  `gh issue list --milestone "M25 Governance + Decomposition + Velocity" --state open ...`
- Governance verification suites:
  `scripts/dev/test-roadmap-status-sync.sh`
  `python3 .github/scripts/test_roadmap_status_workflow_contract.py`
  `scripts/dev/roadmap-status-sync.sh --check --quiet`

## ADR References

- Not required.
