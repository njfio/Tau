# Issue 1652 Plan

Status: Reviewed

## Approach

1. Use `tasks/reports/m23-rustdoc-marker-count.json` as current marker source.
2. Author quota allocation artifact:
   - `tasks/policies/m23-doc-allocation-plan.json`
   - crate-level `current/target/delta`
   - checkpoint schedule and escalation workflow
3. Publish operator-readable summary:
   - `tasks/reports/m23-doc-allocation-plan.md`
   - `docs/guides/doc-density-allocation-plan.md`
4. Add contract test to validate math, cadence mapping, and docs references.

## Affected Areas

- `tasks/policies/m23-doc-allocation-plan.json`
- `tasks/reports/m23-doc-allocation-plan.md`
- `docs/guides/doc-density-allocation-plan.md`
- `docs/guides/doc-density-scorecard.md`
- `.github/scripts/test_doc_allocation_plan_contract.py`

## Risks And Mitigations

- Risk: allocation math drift after manual edits
  - Mitigation: contract test enforces sum/delta invariants.
- Risk: checkpoint plans become stale
  - Mitigation: include explicit schedule and docs references for periodic review.

## ADR

No architecture/dependency/protocol change. ADR not required.
