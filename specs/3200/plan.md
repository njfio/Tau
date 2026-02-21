# Plan: Issue #3200 - reconcile stale dashboard gap claim in whats-missing report

## Approach
1. Use `scripts/dev/verify-dashboard-consolidation.sh` as evidence baseline.
2. Apply RED by updating `scripts/dev/test-whats-missing.sh` to require resolved dashboard marker and reject stale marker.
3. Update `tasks/whats-missing.md` with resolved dashboard entry and remaining-gap section reconciliation.
4. Re-run conformance scripts and fmt/clippy checks.

## Affected Modules
- `tasks/whats-missing.md`
- `scripts/dev/test-whats-missing.sh`
- `specs/milestones/m227/index.md`
- `specs/3200/spec.md`
- `specs/3200/plan.md`
- `specs/3200/tasks.md`

## Risks & Mitigations
- Risk: over-claiming consolidation beyond verified scope.
  - Mitigation: tie resolved marker directly to `verify-dashboard-consolidation.sh` and ADR file evidence.
- Risk: report churn causes marker drift.
  - Mitigation: explicit script assertions on new/stale markers.

## Interfaces / Contracts
- Report marker contract in `tasks/whats-missing.md`.
- Conformance assertions in `scripts/dev/test-whats-missing.sh`.

## ADR
No ADR required (report/conformance reconciliation only).
