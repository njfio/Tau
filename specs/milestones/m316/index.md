# M316 - True RL operations drill depth verification wave

Status: Active

## Context
M316 deepens true RL productionization verification by adding one deterministic
gate that aggregates M24 operational safety proof, resume-after-crash
playbook validation, live benchmark proof, and rollback drill contracts into a
single auditable report.

Primary sources:
- `scripts/verify/m310-rl-policy-ops-depth.sh`
- `scripts/demo/m24-rl-operational-safety-proof.sh`
- `scripts/demo/test-m24-rl-resume-after-crash-playbook.sh`
- `docs/guides/training-ops.md`

## Issue Hierarchy
- Epic: #3520
- Story: #3521
- Task: #3522

## Scope
- Add deterministic M316 true RL operations drill-depth script and report.
- Add script contract test with fail-closed required-step checks.
- Map M24 operational safety/resume/benchmark/rollback contracts to selectors.
- Update README links with M316 verification entrypoint.

## Exit Criteria
- `specs/3522/spec.md` is `Implemented` with AC evidence.
- M316 report includes all required true RL drill-depth step IDs.
- Contract test fails closed on missing required-step IDs.
- README true RL gap entry includes M316 verification entrypoint.
