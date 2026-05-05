# Spec: Issue #3759 - Harness static preview action guard

Status: Implemented

## Problem Statement

The `/ops/harness` operator UI can be rendered as a standalone static HTML file for browser review. In that mode, submitting a harness POST form navigates the browser from the harness page to a missing `file:///ops/...` path, replacing the UI with an error page. Local visual review should not destroy the screen, while the gateway-backed form routes must remain intact for real deployments.

## Scope

In scope:

- Add a harness-scoped static preview guard for POST form submissions when the page is opened from `file://`.
- Preserve existing form IDs, methods, actions, button semantics, and data markers for gateway execution.
- Expose testable DOM markers for the preview guard and blocked preview state.
- Verify the behavior with a rendered static HTML file in the in-app browser.

Out of scope:

- Implementing backend handlers for benchmark or proposal actions.
- Changing gateway action routes.
- Adding a client-side framework or new dependency.
- Intercepting normal HTTP/HTTPS gateway submissions.

## Acceptance Criteria

### AC-1 Static preview submissions stay on the harness screen

Given the harness page is opened as a standalone `file://` preview, when an operator activates a POST action such as Run Benchmark, then the browser remains on the rendered harness page and records that a preview submission was blocked.

### AC-2 Gateway form contracts are preserved

Given the harness page is rendered for the real gateway, when the DOM is inspected, then the existing benchmark and self-improvement POST form actions, methods, and buttons remain unchanged.

### AC-3 Preview guard is scoped and auditable

Given the harness route is rendered, when the DOM is inspected, then the preview guard has explicit markers showing it only guards `file://` POST forms and does not require external dependencies.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | harness HTML opened as `file://` | submit benchmark form | current URL remains the rendered file and blocked state is marked |
| C-02 | AC-2 | Functional | harness route context | render shell | benchmark and proposal forms keep existing POST actions and button ids |
| C-03 | AC-3 | Functional | rendered harness shell | inspect DOM contract | preview guard script and status marker expose file-protocol-only behavior |

## Success Metrics / Observable Signals

- Dashboard UI tests prove the static preview guard contract.
- Existing harness action contract tests remain green.
- Browser Use confirms local `file://` preview no longer navigates to `file:///ops/harness/run-benchmark`.
- No new dependency is introduced.
