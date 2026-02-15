# Issue 1710 Plan

Status: Reviewed

## Approach

1. Add `scripts/demo/m24-rl-operational-safety-proof.sh` to execute a fixed
   check list:
   - prompt-optimization control command suite
   - checkpoint-promotion safety gate suite
   - resume-after-crash playbook validator suite
2. Emit deterministic JSON + Markdown artifacts under `tasks/reports/`.
3. Include runbook evidence metadata for
   `docs/guides/prompt-optimization-recovery-runbook.md`.
4. Add `scripts/demo/test-m24-rl-operational-safety-proof.sh` with:
   - passing mock runner path
   - failing mock runner path (fail-closed assertion)
5. Update docs to point operators at the new proof command.

## Affected Areas

- `scripts/demo/m24-rl-operational-safety-proof.sh` (new)
- `scripts/demo/test-m24-rl-operational-safety-proof.sh` (new)
- `docs/guides/training-ops.md`
- `docs/README.md`
- `specs/1710/spec.md`
- `specs/1710/plan.md`
- `specs/1710/tasks.md`

## Risks And Mitigations

- Risk: proof script runtime is too slow for local iteration.
  - Mitigation: support mock runner hook for script tests; run live checks only
    for operator proof runs.
- Risk: artifact schema drift breaks downstream consumers.
  - Mitigation: enforce schema in script test assertions.
- Risk: runbook link drift.
  - Mitigation: explicit runbook existence check in proof output.

## ADR

No architecture/protocol/dependency change; ADR not required.
