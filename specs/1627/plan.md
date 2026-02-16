# Issue 1627 Plan

Status: Reviewed

## Approach

1. Add rubric policy JSON:
   - `tasks/policies/scaffold-merge-remove-rubric.json`
   - criteria weights, score scale, decision thresholds, unresolved policy
2. Add scoring-sheet generator script:
   - `scripts/dev/scaffold-merge-remove-decision-matrix.sh`
   - emits JSON + Markdown artifacts under `tasks/reports/`
3. Add schema and generated artifacts:
   - `tasks/schemas/m21-scaffold-merge-remove-decision-matrix.schema.json`
   - `tasks/reports/m21-scaffold-merge-remove-decision-matrix.{json,md}`
4. Add tests-first harness:
   - `scripts/dev/test-scaffold-merge-remove-decision-matrix.sh`
   - validates AC/C-01..C-04 plus fail-closed regression behavior

## Affected Areas

- `scripts/dev/scaffold-merge-remove-decision-matrix.sh`
- `scripts/dev/test-scaffold-merge-remove-decision-matrix.sh`
- `tasks/policies/scaffold-merge-remove-rubric.json`
- `tasks/schemas/m21-scaffold-merge-remove-decision-matrix.schema.json`
- `tasks/reports/m21-scaffold-merge-remove-decision-matrix.json`
- `tasks/reports/m21-scaffold-merge-remove-decision-matrix.md`
- `specs/1627/spec.md`
- `specs/1627/plan.md`
- `specs/1627/tasks.md`

## Risks And Mitigations

- Risk: candidate list drifts from lane scope.
  - Mitigation: embed deterministic default candidate set with explicit IDs and owners.
- Risk: scoring ambiguity allows unresolved actions.
  - Mitigation: enforce `unresolved_allowed=false` and fail closed.
- Risk: output drift across runs.
  - Mitigation: deterministic sorting and fixed generated-at override support.

## ADR

No new dependencies or protocol changes; ADR not required.
