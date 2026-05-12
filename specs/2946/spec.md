# Spec: Issue #2946 - Accessibility contract markers and conformance tests

Status: Implemented

## Problem Statement
Tau Ops PRD accessibility acceptance items (`2155-2159`) need explicit, testable dashboard contracts. The current shell includes partial accessibility attributes, but it lacks a dedicated, deterministic accessibility contract surface covering keyboard navigation, live region announcements, focus indicators, and reduced-motion behavior.

## Acceptance Criteria

### AC-1 Accessibility conformance marker surface exists
Given the Tau Ops shell,
When accessibility contracts are inspected,
Then output includes a dedicated accessibility contract marker section.

### AC-2 Keyboard navigation contracts are declared
Given shell navigation controls,
When keyboard support contracts are inspected,
Then output includes skip-link and keyboard navigation markers.

### AC-3 Dynamic content announcement contracts are declared
Given live-updating dashboard content,
When screen-reader announcement contracts are inspected,
Then output includes ARIA live region markers.

### AC-4 Focus indicator contracts are declared
Given interactive controls,
When focus visibility contracts are inspected,
Then output declares focus-ring marker contracts.

### AC-5 Reduced-motion contracts are declared
Given user motion preferences,
When motion behavior contracts are inspected,
Then output declares reduced-motion compliance markers.

### AC-6 Disclosure indicators do not emit textual pseudo-content
Given collapsed chat and memory disclosure controls render,
When accessibility tree text is inspected,
Then decorative disclosure indicators do not contribute `Open`/`Close` pseudo-text to the readable page content.

## Scope

### In Scope
- Add deterministic accessibility contract markers in `tau-dashboard-ui` SSR shell.
- Add conformance tests for PRD items `2155-2159`.
- Keep disclosure toggle state indicators decorative so they do not pollute screen-reader/browser snapshots with repeated `Open`/`Close` text.
- Preserve existing route/panel behaviors.

### Out of Scope
- Full runtime axe-core execution in CI.
- CSS design overhaul beyond marker declarations.
- Browser-side animation implementation.

## Conformance Cases
- C-01: accessibility contract section marker exists.
- C-02: keyboard navigation markers (skip-link + nav contract) exist.
- C-03: live-region markers exist for dynamic announcements.
- C-04: focus indicator contract markers exist.
- C-05: reduced-motion contract markers exist.
- C-06: disclosure indicator CSS does not emit literal `Open` or `Close` pseudo-content.

## Success Metrics / Observable Signals
- `cargo test -p tau-dashboard-ui -- --test-threads=1` passes with new accessibility conformance tests.
- Existing dashboard suite remains green.

## Approval Gate
P1 single-module scope. Spec is agent-reviewed and proceeds under the user’s explicit instruction to continue end-to-end contract execution.
