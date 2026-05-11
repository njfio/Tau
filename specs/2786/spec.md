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

### AC-7 No-auth login does not show a credential field
Given the gateway maps `localhost-dev` auth to `ui_auth_mode=none`,
When an operator opens `/ops/login`,
Then the visible login shell uses a no-auth heading/status and hides the
password/token input from the operator surface.

### AC-8 Login skip navigation targets the visible shell
Given an operator opens `/ops/login`,
When keyboard navigation uses the skip-to-main link,
Then the link targets the visible login shell rather than the hidden protected
operations shell.

### AC-9 No-auth Continue exposes navigation semantics
Given localhost-dev mode renders no credential challenge,
When an operator or assistive technology reads the Continue control,
Then Continue is exposed as a link with the `/ops` href instead of a button role.

### AC-10 Login route does not ship hidden protected payload
Given an operator opens `/ops/login`,
When the server renders the login shell,
Then the hidden protected shell is marked as pruned and omits protected
route payloads such as chat, sessions, memory, harness, command center, and
deploy panels.

### AC-11 Login route narrows visible navigation before Continue
Given an operator opens `/ops/login`,
When the visible shell renders,
Then the sidebar is scoped to login, protected navigation rows are hidden from
the operator rail, and the breadcrumb root preserves `/ops/login` context with
a login label instead of advertising a fake Home action that bypasses Continue.

## Scope

### In Scope
- New gateway auth bootstrap endpoint.
- Auth-aware shell context and markers in `tau-dashboard-ui`.
- Gateway route wiring for `/ops/login` and context-aware `/ops` rendering.
- Working no-auth Continue navigation for localhost-dev login shells.
- Auth-mode-specific login help copy for the no-auth localhost-dev shell.
- No-auth heading/status markers and hidden credential field rendering.
- Route-aware skip-to-main target for the login shell.
- Link semantics for the no-auth Continue navigation control.
- Protected payload pruning for the login route.
- Login-scoped navigation rail and route-aware breadcrumb behavior on
  `/ops/login`.
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
- C-08 (regression): localhost-dev `/ops/login` hides the credential input and
  renders an access-ready heading plus no-auth status marker.
- C-09 (regression): `/ops/login` renders a skip-to-main link targeting the
  visible login shell.
- C-10 (regression): localhost-dev `/ops/login` exposes Continue as a link
  with a preserved-context `/ops` href and no button role.
- C-11 (regression): `/ops/login` keeps the hidden protected shell as a
  pruned marker and omits hidden protected route payload panels.
- C-12 (regression): `/ops/login` exposes a login-scoped sidebar, hides
  protected route rows in the visible rail, and labels the breadcrumb root as
  login while keeping it on the login route.

## Success Metrics / Observable Signals
- `cargo test -p tau-dashboard-ui -- --test-threads=1` passes with new auth/route marker coverage.
- `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1` passes.
- Browser proof confirms the visible Continue control on a live
  localhost-dev `/ops/login` tab navigates to `/ops` with the same shell
  context.
- Browser proof confirms live localhost-dev login copy says no credential is
  required.
- Browser proof confirms live localhost-dev login no longer exposes a visible
  password/token input.
- Browser proof confirms live login skip navigation points to the visible login
  shell.
- Browser proof confirms live localhost-dev Continue is exposed as a link with
  a URL in the browser tree.
- Browser proof confirms live `/ops/login` no longer ships hidden protected
  route panels in the document payload.
- Browser proof confirms live `/ops/login` shows only route-appropriate Login
  navigation before Continue, while the protected route rows are hidden.
- Existing dashboard/auth regression tests continue to pass.

## Approval Gate
P1 multi-module slice proceeds with spec marked `Reviewed` per AGENTS.md self-acceptance rule.
