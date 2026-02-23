# Spec: Issue #3426 - Full auth lifecycle verification and hardening

Status: Implemented

## Problem Statement
Auth flows are implemented across gateway token/password-session modes and provider credential commands, but full lifecycle verification is incomplete and some gateway auth contracts are not explicitly tested. We need deterministic conformance coverage and targeted hardening so auth behavior is fail-closed and operationally observable.

## Scope
In scope:
- Gateway auth bootstrap/session/status lifecycle conformance for token, password-session, and localhost-dev modes.
- Verification of auth-session negative paths (mode mismatch, malformed body, invalid credentials, expiry).
- Auth header parsing hardening where current behavior is unnecessarily strict.
- Gateway auth status metrics verification for issued sessions, active sessions, and auth failures.

Out of scope:
- Adding new auth endpoints or changing external auth wire schemas.
- Provider protocol changes beyond test/regression coverage.
- OAuth provider flow redesign.

## Acceptance Criteria
### AC-1 Bootstrap auth contract is verified for all gateway auth modes
Given gateway auth modes `token`, `password-session`, and `localhost-dev`,
when calling `GET /gateway/auth/bootstrap`,
then `auth_mode`, `ui_auth_mode`, and `requires_authentication` are correct for each mode.

### AC-2 Password-session lifecycle is verified end-to-end with fail-closed negatives
Given password-session mode,
when issuing a session token and calling protected endpoints,
then valid tokens authorize requests, expired tokens fail closed, and invalid credential/malformed/mode-mismatch paths return deterministic errors.

### AC-3 Bearer auth parsing is hardened without relaxing security
Given authenticated gateway routes,
when clients send valid bearer credentials with scheme casing variance,
then valid credentials are accepted and invalid/missing credentials remain unauthorized.

### AC-4 Auth status telemetry reflects lifecycle behavior
Given a sequence of successful and failed auth interactions,
when reading `GET /gateway/status`,
then auth report fields (`active_sessions`, `total_sessions_issued`, `auth_failures`) reflect observed behavior.

### AC-5 Spec-driven artifact and test mapping is complete
Given AGENTS lifecycle requirements,
when reviewing issue artifacts,
then `spec.md`, `plan.md`, and `tasks.md` exist with conformance cases mapped to tests and verification commands.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | token mode | call bootstrap endpoint | reports `auth_mode=token`, `ui_auth_mode=token`, `requires_authentication=true` |
| C-02 | AC-1 | Functional/Conformance | password-session mode | call bootstrap endpoint | reports `auth_mode=password-session`, `ui_auth_mode=password-session`, `requires_authentication=true` |
| C-03 | AC-1 | Functional/Conformance | localhost-dev mode | call bootstrap endpoint | reports `ui_auth_mode=none`, `requires_authentication=false` |
| C-04 | AC-2 | Functional/Integration | password-session mode | issue session token + call protected endpoint | protected request succeeds |
| C-05 | AC-2 | Regression | password-session mode | use expired token | protected request returns unauthorized |
| C-06 | AC-2 | Regression | non-password auth mode | call auth-session endpoint | returns deterministic `auth_mode_mismatch` error |
| C-07 | AC-2 | Regression | password-session mode | submit malformed session JSON | returns deterministic `malformed_json` error |
| C-08 | AC-3 | Functional/Regression | token mode | call protected route with lowercase `bearer` scheme + valid token | request succeeds |
| C-09 | AC-3 | Regression | token mode | call protected route with missing/invalid token | request remains unauthorized |
| C-10 | AC-4 | Functional/Conformance | mixed auth successes/failures | inspect gateway status | auth telemetry counters match expected values |
| C-11 | AC-5 | Conformance | issue artifacts | verify files/sections | spec/plan/tasks + mappings exist |

## Success Metrics / Observable Signals
- New gateway auth conformance tests pass deterministically.
- No regression in existing auth/session/OpenAI compatibility tests.
- Auth status counters are explicitly validated by tests, not implicit.
