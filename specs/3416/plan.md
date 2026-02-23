# Plan: Issue #3416 - README clarity refresh

## Approach
1. Build a feature inventory from:
   - workspace crate membership (`Cargo.toml`, `crates/`),
   - operator/docs index (`docs/guides`, `docs/architecture`),
   - runnable scripts (`scripts/dev`, `scripts/demo`).
2. Run RED checks that assert missing onboarding structure in current README.
3. Rewrite `README.md` with:
   - concise value proposition,
   - capability summary with realistic boundaries,
   - 5-minute quickstart,
   - common workflows and doc map.
4. Add integration transparency sections:
   - integrated end-to-end paths that work now,
   - a maturity matrix by capability area,
   - a gap/next-step table linking to concrete runbooks/plans for True RL, dashboard, auth verification, and TUI.
5. Run GREEN conformance checks on:
   - required headings,
   - referenced links/scripts existence,
   - command entrypoints present in workspace.
   - docs helper unittest suite used by CI.

## Affected Modules
- `README.md`
- `specs/milestones/m294/index.md`
- `specs/3416/spec.md`
- `specs/3416/plan.md`
- `specs/3416/tasks.md`

## Risks / Mitigations
- Risk: overstating feature maturity.
  - Mitigation: keep capability language tied to current code/docs and known status sections.
- Risk: broken links or command drift.
  - Mitigation: run explicit path/command existence checks after rewrite.

## Interfaces / Contracts
- User-facing docs contract only; no runtime API changes.

## ADR
- Not required (documentation-only change).
