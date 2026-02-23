# Plan: Issue #3406 - M291 milestone state closure

## Approach
1. Verify milestone 291 currently has `open_issues=0`.
2. Close milestone 291 using GitHub API.
3. Update `specs/milestones/m291/index.md` closeout with explicit closure metadata.
4. Verify milestone API state and docs consistency.

## Affected Modules
- `specs/milestones/m291/index.md`

## Risks / Mitigations
- Risk: accidental closure of wrong milestone.
  - Mitigation: use explicit milestone number (`291`) and verify title in response.

## Interfaces / Contracts
- Governance metadata only; no runtime/API behavior changes.

## ADR
- Not required.
