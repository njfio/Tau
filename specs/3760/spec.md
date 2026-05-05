# Spec: Issue #3760 - Ops shell static preview navigation guard

Status: Implemented

## Problem Statement

The Tau Ops shell can be reviewed as standalone SSR HTML in the in-app browser. In that mode, absolute internal links such as `/ops/agents` navigate to missing `file:///ops/...` paths and replace the UI with a browser error page. Static preview clicks should not destroy the review surface, while real gateway navigation links must stay intact.

## Scope

In scope:

- Add an ops-shell static preview guard for absolute internal links when opened from `file://`.
- Preserve existing sidebar, breadcrumb, harness, and gateway route `href` values.
- Mark blocked preview link clicks in the DOM for verification.
- Verify the behavior with Browser Use against a rendered standalone HTML file.

Out of scope:

- Client-side route switching in static preview mode.
- Renaming or changing route paths.
- Adding dependencies or a client-side router.
- Intercepting HTTP/HTTPS gateway navigation.

## Acceptance Criteria

### AC-1 Static preview internal links do not leave the rendered shell

Given the ops shell is opened as a standalone `file://` preview, when an operator clicks an absolute internal route link such as Agent Fleet, then the browser stays on the rendered file and the clicked link is marked as blocked for preview.

### AC-2 Gateway navigation contracts are preserved

Given the shell is rendered for the gateway, when the DOM is inspected, then existing sidebar and breadcrumb `href` values remain unchanged.

### AC-3 Link guard is scoped and auditable

Given the shell is rendered, when the DOM is inspected, then the link guard has explicit markers showing it only guards `file://` absolute local links inside `#tau-ops-shell`.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | shell HTML opened as `file://` | click Agent Fleet | current URL remains the rendered file and link is marked blocked |
| C-02 | AC-2 | Functional | default shell render | inspect nav links | `/ops/agents`, `/ops/harness`, and `/ops` hrefs remain unchanged |
| C-03 | AC-3 | Functional | rendered shell | inspect DOM contract | preview link guard markers expose file-protocol-only absolute-route behavior |

## Success Metrics / Observable Signals

- Dashboard UI tests prove the static preview link guard contract.
- Existing harness form guard tests remain green.
- Browser Use confirms local `file://` preview no longer navigates to `file:///ops/agents`.
- No new dependency is introduced.
