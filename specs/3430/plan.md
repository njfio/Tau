# Plan: Issue #3430 - M296 GA readiness gate and closeout

## Approach
1. Add milestone/spec governance artifacts for M296 and issue `#3430`.
2. Write RED coverage for a new GA verification script:
   - deterministic pass/fail report semantics,
   - fail-closed exit behavior,
   - rollback trigger matrix fields and closeout summary fields.
3. Implement `scripts/verify/m296-ga-readiness-gate.sh`:
   - execute required checks,
   - persist per-step logs,
   - produce deterministic JSON report.
4. Update operator-facing docs:
   - README command path,
   - docs index entrypoint,
   - dedicated GA readiness runbook and ownership mapping.
5. Run scoped verification commands and capture RED/GREEN evidence in `tasks.md`.

## Affected Modules
- `scripts/verify/m296-ga-readiness-gate.sh` (new)
- `scripts/verify/test-m296-ga-readiness-gate.sh` (new)
- `README.md`
- `docs/README.md`
- `docs/guides/m296-ga-readiness-gate.md` (new)
- `docs/guides/ops-readiness-live-validation.md`
- `docs/guides/consolidated-runtime-rollback-drill.md`
- `docs/guides/runbook-ownership-map.md`
- `specs/milestones/m296/index.md`
- `specs/3430/spec.md`
- `specs/3430/plan.md`
- `specs/3430/tasks.md`

## Risks / Mitigations
- Risk: GA gate script drifts from underlying phase checks.
  - Mitigation: call existing verification scripts/tests as source-of-truth checks.
- Risk: report shape becomes ambiguous for operators.
  - Mitigation: include fixed schema fields (`suite_id`, `overall`, `signoff_criteria`, `steps`, `closeout_summary`).
- Risk: docs drift from executable commands.
  - Mitigation: link docs directly to script paths and include explicit command snippets.

## Interfaces / Contracts
- New command: `scripts/verify/m296-ga-readiness-gate.sh`
- New artifact: `artifacts/operator-ga-readiness/verification-report.json`
- Signoff criteria contract:
  - readiness/auth/RL checks green,
  - rollback contract checks green,
  - docs linkage checks green,
  - milestone closeout artifact references present.

## ADR
- Not required: no new dependency, no protocol/schema migration for runtime APIs.
