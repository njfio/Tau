# Plan: Deterministic Full Validation Target

GitHub Issue: #3763

## Approach

Keep doctests enabled and move the release validation environment away from the
shared default `target/` directory. Add a small full-mode setup step to
`scripts/dev/fast-validate.sh`: if the caller did not provide
`CARGO_TARGET_DIR`, export a deterministic branch-scoped target directory under
`/tmp`; if the caller did provide one, leave it untouched. Export
`CARGO_INCREMENTAL=0` for full validation when unset so release checks avoid
incremental compiler state.

## Affected Modules

- `scripts/dev/fast-validate.sh`
- `scripts/dev/test-fast-validate.sh`
- `tasks/reports/full-workspace-doc-test-overrun.md`
- `specs/3800/*`

## Risks

- Full validation can take longer from an isolated target. Mitigation: apply the
  isolated target only to full-workspace validation, not every scoped edit loop.
- A caller may need a custom target directory for CI or local caching.
  Mitigation: preserve any explicit `CARGO_TARGET_DIR`.
- The root cause could still be a local corrupted default target. Mitigation:
  document the default-target timeout separately from the isolated-target pass.

## Verification

Run the fast-validate script tests first, then rerun the focused
`tau-agent-core` doctest gate in an isolated target. Finish by running
`scripts/dev/fast-validate.sh --full` with the same release-grade environment.
