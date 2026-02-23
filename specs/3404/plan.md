# Plan: Issue #3404 - M291 closeout doc alignment

## Approach
1. Update only `specs/milestones/m291/index.md`:
   - `Status: In Progress` -> completed status.
   - Phase 9 line -> delivered wording.
   - add explicit final conformance signal line.
2. Validate the conformance summary still computes as expected from `specs/3386/conformance-matrix.md`.

## Affected Modules
- `specs/milestones/m291/index.md`

## Risks / Mitigations
- Risk: mismatch between stated summary and live matrix counts.
  - Mitigation: compute counts directly via deterministic `awk` command before closeout.

## Interfaces / Contracts
- Documentation-only change; no runtime/API contract changes.

## ADR
- Not required.
