# M312 - Auth live-env validation depth verification wave

Status: Active

## Context
M312 deepens auth validation by adding deterministic contract coverage for
`m296-live-auth-validation` skip/enable/key-handling behavior and aggregating
live-auth, auth-depth, and credential-lifecycle gates into one auditable suite.

Primary sources:
- `scripts/verify/m296-live-auth-validation.sh`
- `scripts/verify/m303-auth-workflow-depth.sh`
- `scripts/verify/m309-auth-credential-lifecycle-depth.sh`
- `docs/provider-auth/provider-auth-capability-matrix.md`

## Issue Hierarchy
- Epic: #3504
- Story: #3505
- Task: #3506

## Scope
- Add deterministic contract test for `m296-live-auth-validation.sh`.
- Add deterministic M312 auth live-env depth verification script and report.
- Add M312 script contract test with fail-closed required-step checks.
- Update README links with M312 entrypoint.

## Exit Criteria
- `specs/3506/spec.md` is `Implemented` with AC evidence.
- M312 script report includes all required auth live-env step IDs.
- Contract tests fail closed on missing required-step IDs and invalid report.
- README includes M312 verification entrypoint.
