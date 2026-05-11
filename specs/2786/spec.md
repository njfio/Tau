# Spec: Issue #2786 - PRD Phase 1B auth bootstrap and protected route shell markers

Status: Implemented

## Problem Statement
Phase 1A introduced a Leptos SSR shell at `/ops`, but the shell lacks explicit auth mode and login/protected route contracts. The dashboard cannot deterministically bootstrap auth behavior (token vs password-session vs no-auth) without parsing unrelated endpoints, and there is no dedicated login route marker contract.

## Acceptance Criteria

### AC-1 Gateway exposes an unauthenticated auth bootstrap contract endpoint
Given gateway runtime auth mode configuration,
When requesting `GET /gateway/auth/bootstrap`,
Then response includes deterministic auth metadata for dashboard bootstrap (`auth_mode`, `ui_auth_mode`, `requires_authentication`) plus auth/login route endpoint fields, without leaking token/password secrets.

### AC-2 `tau-dashboard-ui` shell renders auth and route markers
Given `tau-dashboard-ui` shell render entrypoint,
When rendering with explicit auth/route context,
Then output contains deterministic markers for auth mode, login-required state, login shell section, and protected command-center section.

### AC-3 Gateway serves auth-aware shell routes for `/ops` and `/ops/login`
Given gateway router integration,
When requesting `/ops` or `/ops/login`,
Then responses are HTML from `tau-dashboard-ui` auth-aware renderer with correct active-route markers and existing component markers preserved.

### AC-4 Existing dashboard and auth session contracts remain stable
Given prior dashboard and auth contracts,
When phase 1B changes are integrated,
Then `/dashboard` shell and `POST /gateway/auth/session` behavior stay unchanged and existing tests continue passing.

### AC-5 Localhost-dev login continue is actionable
Given the gateway maps `localhost-dev` auth to `ui_auth_mode=none`,
When an operator opens `/ops/login`,
Then the visible Continue control navigates to the protected `/ops` shell while
preserving the current theme, sidebar, and session query context. The disabled
auth input is marked as not enabled so the no-auth page does not imply a
password/token is required.

### AC-6 Login help copy matches the active auth mode
Given the gateway maps `localhost-dev` auth to `ui_auth_mode=none`,
When an operator opens `/ops/login`,
Then the help copy explicitly says no credential is required and does not reuse
generic gateway-auth wording that implies an auth action is required.

## Scope

### In Scope
- New gateway auth bootstrap endpoint.
- Auth-aware shell context and markers in `tau-dashboard-ui`.
- Gateway route wiring for `/ops/login` and context-aware `/ops` rendering.
- Working no-auth Continue navigation for localhost-dev login shells.
- Auth-mode-specific login help copy for the no-auth localhost-dev shell.
- Scoped regression tests for dashboard shell and auth session behavior.

### Out of Scope
- Full token/password login submission UX/hydration behavior.
- Token refresh scheduler and WebSocket auth re-auth UX.
- Full 14-view dashboard navigation implementation.

## Conformance Cases
- C-01 (conformance): `GET /gateway/auth/bootstrap` returns expected auth bootstrap fields for token mode.
- C-02 (conformance): auth bootstrap reports `ui_auth_mode=none` and `requires_authentication=false` for localhost-dev mode.
- C-03 (functional): `tau-dashboard-ui` auth-aware SSR shell includes auth and route markers for login/protected sections.
- C-04 (integration): `/ops` and `/ops/login` return auth-aware shell with correct active-route markers.
- C-05 (regression): existing `/dashboard` and `POST /gateway/auth/session` tests remain green.
- C-06 (regression): localhost-dev `/ops/login` renders a Continue control that
  reaches `/ops` with theme/sidebar/session context preserved.
- C-07 (regression): localhost-dev `/ops/login` renders no-auth help copy and
  omits generic gateway-auth instructions.

## Success Metrics / Observable Signals
- `cargo test -p tau-dashboard-ui -- --test-threads=1` passes with new auth/route marker coverage.
- `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1` passes.
- Browser proof confirms the visible Continue control on a live
  localhost-dev `/ops/login` tab navigates to `/ops` with the same shell
  context.
- Browser proof confirms live localhost-dev login copy says no credential is
  required.
- Existing dashboard/auth regression tests continue to pass.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule.
