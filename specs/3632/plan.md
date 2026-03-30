# Plan: Issue #3632 - Make fast-validate resilient to shallow PR history

## Approach
1. Add a deterministic shell regression test that sets up a shallow clone with
   a fetched base commit but no local merge base.
2. Update `collect_changed_files()` so it:
   - still uses `base...HEAD` when possible,
   - falls back to `base..HEAD` when the merge base is unavailable locally,
   - emits a bounded warning instead of raw git fatal noise,
   - only forces full-workspace scope when neither diff path is usable.
3. Re-run the fast-validate test script to prove the regression is fixed.

## Affected Areas
- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`
- `specs/milestones/m330/index.md`
- `specs/3632/`

## Compatibility / Contract Notes
- No runtime product behavior changes.
- CI validation semantics change only for the missing-merge-base edge case.
- The fallback should remain conservative: it uses `base..HEAD` only when the
  base commit object exists locally but merge-base discovery fails.

## Risks / Mitigations
- Risk: the fallback could under- or over-report changed files in a non-ancestor
  case.
  Mitigation: only use two-dot fallback after verifying the base commit exists
  and only for the explicit missing-merge-base path.
- Risk: script tests become brittle because of temp-repo setup.
  Mitigation: keep the fixture minimal and fully self-contained in a temp dir.

## Verification
- `./scripts/dev/test-fast-validate.sh`
- Optional local spot check:
  `./scripts/dev/fast-validate.sh --check-only --skip-fmt --base <sha>`
  in the temp shallow-history fixture
