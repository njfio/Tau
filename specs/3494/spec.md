# Spec: Issue #3494 - M309 deterministic auth credential lifecycle depth gate

Status: Implemented

## Problem Statement
Auth verification currently spans multiple suites/scripts, but operators still
lack one deterministic gate focused on credential lifecycle depth
(integration-auth set/status/rotate/revoke plus resolve-secret fail-closed
contracts) alongside provider and gateway auth lifecycle selectors.

## Scope
In scope:
- Add `scripts/verify/m309-auth-credential-lifecycle-depth.sh`.
- Add `scripts/verify/test-m309-auth-credential-lifecycle-depth.sh`.
- Emit deterministic JSON report artifact with required-step inventory.
- Map provider/gateway/integration-auth/secret-resolution contracts to
  executable selectors.
- Update README links with M309 verification entrypoint.

Out of scope:
- New auth protocols/endpoints.
- Live third-party auth validation behavior changes.
- Credential-store schema changes.

## Acceptance Criteria
### AC-1 Deterministic M309 script emits auth credential-lifecycle report
Given local execution,
when `scripts/verify/m309-auth-credential-lifecycle-depth.sh` runs,
then it executes required selectors and writes a deterministic report artifact
with per-step status.

### AC-2 Auth credential lifecycle contracts are explicitly mapped and executable
Given M309 step inventory,
when script selectors execute,
then coverage includes provider auth conformance, gateway auth-session
lifecycle, integration-auth set/status/rotate/revoke lifecycle, and
resolve-secret fail-closed behavior.

### AC-3 Contract test enforces fail-closed required-step/report consistency
Given `scripts/verify/test-m309-auth-credential-lifecycle-depth.sh`,
when pass/fail/tamper paths run,
then missing required-step IDs or invalid report consistency fails closed.

### AC-4 README references M309 auth credential-lifecycle gate
Given README auth workflow gap entries,
when reviewed,
then M309 script is included as execution entrypoint.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M309 script | run | report artifact emitted with valid schema |
| C-02 | AC-2 | Functional | required-step map | execute script | provider/gateway/integration-auth/resolve-secret selectors run |
| C-03 | AC-2 | Integration | multi-crate auth selectors | run script | lifecycle contracts pass deterministically |
| C-04 | AC-3 | Conformance/Regression | M309 script contract test | pass/fail/tamper execution | fail-closed behavior enforced |
| C-05 | AC-4 | Functional | README links | inspect README | M309 link present |

## Success Metrics / Observable Signals
- One command verifies auth credential lifecycle depth with JSON output.
- Required-step checks detect silent coverage drift.
- README references the auth credential-lifecycle verification gate.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m309-auth-credential-lifecycle-depth.sh` passed and emitted `artifacts/auth-credential-lifecycle-depth/verification-report.json` (`suite_id=m309_auth_credential_lifecycle_depth`). |
| AC-2 | ✅ | Required M309 step mapping covers provider auth conformance, gateway auth-session lifecycle, integration-auth set/status/rotate/revoke + status contracts, and resolve-secret fail-closed behavior. |
| AC-3 | ✅ | `bash scripts/verify/test-m309-auth-credential-lifecycle-depth.sh` passed pass/fail/tamper contract paths and verify-only fail-closed required-step checks. |
| AC-4 | ✅ | `README.md` auth verification capability notes and the extended-auth gap row include `scripts/verify/m309-auth-credential-lifecycle-depth.sh`. |
