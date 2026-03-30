# Plan: Issue #3633 - Repair broken relative links in March 24 planning docs

## Approach
Use the smallest possible docs-only change:

1. Create the missing milestone/spec/task artifacts for `#3633`.
2. Replace broken plan-local markdown targets with correct relative paths.
3. Re-run the docs link checker to prove the two failing links are gone.

## Affected Areas
- `docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md`
- `docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md`
- `specs/milestones/m330/index.md`
- `specs/3633/`

## Compatibility / Contract Notes
- No code paths or runtime behavior change.
- Only markdown link targets are corrected.
- The edited paths must remain valid when resolved relative to files under
  `docs/plans/`.

## Risks / Mitigations
- Risk: fixing only one visible markdown link leaves another duplicate broken.
  Mitigation: search all occurrences of the offending targets in both files
  before editing.
- Risk: unintentional content churn in large planning docs.
  Mitigation: restrict edits to exact link target strings.

## Verification
- `python3 .github/scripts/docs_link_check.py --repo-root .`
- `git diff -- docs/plans/2026-03-24-001-feat-radical-simplification-self-improvement-plan.md docs/plans/2026-03-24-002-feat-wiring-integration-endgame-plan.md`
