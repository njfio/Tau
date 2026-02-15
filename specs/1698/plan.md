# Issue 1698 Plan

Status: Reviewed

## Approach

1. Add an M24 live-run benchmark protocol section to
   `docs/guides/training-ops.md` with:
   - preconditions and deterministic run controls
   - baseline/trained execution flow
   - required artifacts list and naming conventions
   - significance pass/fail criteria
2. Add `scripts/demo/m24-rl-benchmark-proof-template.json` describing canonical
   benchmark proof artifact fields.
3. Add a validator script plus regression test in `scripts/demo/` to enforce
   template contract stability.

## Affected Areas

- `docs/guides/training-ops.md`
- `scripts/demo/m24-rl-benchmark-proof-template.json`
- `scripts/demo/validate-m24-rl-benchmark-proof-template.sh`
- `scripts/demo/test-m24-rl-benchmark-proof-template.sh`

## Risks And Mitigations

- Risk: protocol drift from future benchmark tooling.
  - Mitigation: keep placeholders explicit and reference owning follow-up issues.
- Risk: over-constraining template too early.
  - Mitigation: enforce only required cross-run comparability fields.

## ADR

No architecture/dependency/protocol wire-format change. ADR not required.
