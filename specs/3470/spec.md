# Spec: Issue #3470 - M303 deterministic auth-depth verification gate

Status: Implemented

## Problem Statement
Current verification entrypoints include auth checks, but they do not provide a
single deterministic operator-facing gate that aggregates provider auth matrix
coverage with gateway auth lifecycle and edge-path selectors. This leaves auth
verification depth fragmented across multiple scripts/tests.

## Scope
In scope:
- Add `scripts/verify/m303-auth-workflow-depth.sh` as a deterministic auth-depth
  gate script.
- Include provider and gateway auth selectors covering:
  - token bootstrap contracts,
  - password-session lifecycle contracts,
  - invalid password/mode mismatch/malformed JSON fail-closed paths,
  - lowercase bearer acceptance,
  - session-expiry fail-closed behavior,
  - localhost-dev no-bearer path.
- Add `scripts/verify/test-m303-auth-workflow-depth.sh` script contract test.
- Update README auth-gap execution links to include M303 gate script.

Out of scope:
- New gateway auth endpoints or protocol changes.
- Live provider key validation (already covered by M296 live script when enabled).
- RBAC/multi-tenant auth models.

## Acceptance Criteria
### AC-1 Deterministic auth-depth verification script exists and passes
Given local workspace execution,
when `scripts/verify/m303-auth-workflow-depth.sh` runs,
then it executes the defined auth selector suite and writes a deterministic
verification report artifact with per-step pass/fail status.

### AC-2 Script covers gateway auth lifecycle and edge-path selectors
Given the M303 selector inventory,
when script steps are reviewed and executed,
then lifecycle and edge-path selectors are included for invalid password, mode
mismatch, malformed payload, lowercase bearer, token expiry, and localhost-dev.

### AC-3 Script contract test guards selector inventory and report schema
Given `scripts/verify/test-m303-auth-workflow-depth.sh`,
when it executes,
then it verifies required step IDs and fail-closed behavior for missing step
coverage markers.

### AC-4 README execution links include the M303 auth-depth gate
Given README gap execution links,
when auth workflow expansion entries are reviewed,
then they include the new M303 verification script link.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M303 script | run script | report artifact written with pass status per step |
| C-02 | AC-2 | Functional | script step map | execute script | gateway edge-path selectors are executed and pass |
| C-03 | AC-2 | Integration | gateway auth suite | run targeted selectors | lifecycle + failure paths pass deterministically |
| C-04 | AC-3 | Conformance/Regression | script contract test | run test script | required step IDs enforced; fail-closed behavior validated |
| C-05 | AC-4 | Functional | README auth gap row | inspect links | M303 script link present |

## Success Metrics / Observable Signals
- One command runs deterministic auth-depth verification (`m303` script).
- Report artifact captures per-step auth verification outcomes.
- README auth-gap section references the M303 gate for operator execution.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `CARGO_TARGET_DIR=target-fast bash scripts/verify/m303-auth-workflow-depth.sh` executed all M303 auth-depth selectors and wrote `artifacts/auth-workflow-depth/verification-report.json` (`suite_id=m303_auth_workflow_depth`, `overall=pass`, `steps=11`). |
| AC-2 | ✅ | `scripts/verify/m303-auth-workflow-depth.sh` includes gateway lifecycle/edge selectors: invalid password, mode mismatch, malformed JSON, lowercase bearer, password-session expiry fail-closed, and localhost-dev no-bearer. |
| AC-3 | ✅ | `bash scripts/verify/test-m303-auth-workflow-depth.sh` validates report schema, required step IDs, fail-on-step failure, and verify-only fail-closed behavior when a required step marker is removed. |
| AC-4 | ✅ | `README.md` auth verification and current-gaps execution links now include `scripts/verify/m303-auth-workflow-depth.sh`. |
