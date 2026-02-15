# Issue 1722 Plan

Status: Reviewed

## Approach

1. Extend allocation contract to include owner-domain cadence and checkpoint
   review schedule.
2. Mirror cadence metadata in `docs/guides/doc-density-targets.json` to keep
   targets + ownership overlays in one place.
3. Publish docs guide updates explaining cadence and escalation workflow.
4. Validate with contract test and docs checks.

## Affected Areas

- `tasks/policies/m23-doc-allocation-plan.json`
- `docs/guides/doc-density-targets.json`
- `docs/guides/doc-density-scorecard.md`
- `docs/guides/doc-density-allocation-plan.md`
- `docs/README.md`
- `.github/scripts/test_doc_allocation_plan_contract.py`

## Risks And Mitigations

- Risk: owner-domain metadata drifts between files
  - Mitigation: contract test compares cadence fields across policy/docs.
- Risk: escalation policy ambiguity
  - Mitigation: explicit step list in policy + guide text.

## ADR

No architecture/dependency/protocol change. ADR not required.
