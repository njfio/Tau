# Plan: Issue #3188 - resync stale claims in tasks/whats-missing.md

## Approach
1. Capture repository evidence for each previously listed missing capability.
2. Add RED conformance script assertions against the stale missing markers.
3. Refresh `tasks/whats-missing.md` to current state with implemented vs unresolved sections.
4. Update conformance script expectations to GREEN for the refreshed report.
5. Run formatting/lint verification.

## Affected Modules
- `tasks/whats-missing.md`
- `scripts/dev/test-whats-missing.sh`
- `specs/milestones/m224/index.md`
- `specs/3188/spec.md`
- `specs/3188/plan.md`
- `specs/3188/tasks.md`

## Risks & Mitigations
- Risk: false positives from text-only checks.
  - Mitigation: validate stable, specific markers rather than fragile prose.
- Risk: report drifts again as features evolve.
  - Mitigation: keep deterministic script as a fast contract gate.

## Interfaces / Contracts
- Documentation contract for `tasks/whats-missing.md` status sections.
- Script contract in `scripts/dev/test-whats-missing.sh`.

## ADR
No ADR required (docs + verification contract refresh only).
