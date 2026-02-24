# Spec: Issue #3506 - M312 deterministic auth live-env validation depth gate

Status: Implemented

## Problem Statement
Live-auth validation currently depends on environment conditions and lacks
dedicated deterministic contract tests for skip/enable/key handling behavior.
Operators also lack one aggregated gate that combines live-auth validation
contracts with existing auth-depth and credential-lifecycle depth suites.

## Scope
In scope:
- Add `scripts/verify/test-m296-live-auth-validation.sh`.
- Add `scripts/verify/m312-auth-live-env-depth.sh`.
- Add `scripts/verify/test-m312-auth-live-env-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Update README links with M312 verification entrypoint.

Out of scope:
- New auth providers or protocol changes.
- Live third-party credential network behavior changes.
- Credential-store schema changes.

## Acceptance Criteria
### AC-1 M296 live-auth validation script has deterministic contract coverage
Given contract test execution,
when `scripts/verify/test-m296-live-auth-validation.sh` runs,
then it validates disabled/missing-key/no-live-key skip behavior and mock-mode
pass/fail behavior deterministically.

### AC-2 Deterministic M312 script emits auth live-env depth report
Given local execution,
when `scripts/verify/m312-auth-live-env-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-3 M312 contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m312-auth-live-env-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M312 auth live-env depth gate
Given README auth verification/gap entries,
when reviewed,
then M312 script is included as execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance/Regression | `test-m296-live-auth-validation.sh` | run | skip/pass/fail contracts enforced deterministically |
| C-02 | AC-2 | Conformance | M312 script | run | report artifact emitted with expected schema |
| C-03 | AC-2 | Integration | required-step map | run script | live-auth + auth-depth + credential-lifecycle selectors run |
| C-04 | AC-3 | Conformance/Regression | `test-m312-auth-live-env-depth.sh` | pass/fail/tamper | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M312 link present |

## Success Metrics / Observable Signals
- Deterministic tests cover live-auth validation skip/pass/fail paths.
- One command verifies auth live-env depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the M312 verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `bash scripts/verify/test-m296-live-auth-validation.sh` passed deterministic disabled/missing-key/no-live-key skip checks plus mock pass/fail checks. |
| AC-2 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m312-auth-live-env-depth.sh` passed and emitted `artifacts/auth-live-env-depth/verification-report.json` (`suite_id=m312_auth_live_env_depth`). |
| AC-3 | ✅ | `bash scripts/verify/test-m312-auth-live-env-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` auth verification capability notes and extended-auth gap row include `scripts/verify/m312-auth-live-env-depth.sh`. |
