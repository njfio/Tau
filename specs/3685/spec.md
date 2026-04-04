# Spec: Issue #3685 - Add command palette paging and full browsing to `tau-tui` REPL

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports autocomplete, history, aliases, and
feedback, but it still only renders the first few matches. When more commands
match than fit in the overlay, the operator cannot reliably inspect or select
deeper results. A stronger REPL launcher needs paging and selection visibility
so keyboard navigation works across the full result set rather than only the
top slice.

## Scope
In scope:
- add windowed command palette rendering for larger result sets
- keep the selected command visible while moving through results
- support stronger keyboard navigation for first/last/page movement
- surface which results are currently shown within the full match count
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3685/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- mouse-based result selection
- fuzzy ranking changes
- dynamic command registration
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 The command palette can browse past the first visible result page
Given the command palette has more matches than fit in its visible list,
when the operator navigates downward,
then Tau keeps the selected command visible and renders the appropriate result
window instead of freezing on the first page.

### AC-2 The command palette supports stronger browsing keys
Given the operator is in the command palette,
when they use page and boundary navigation keys,
then Tau jumps predictably across the result set without losing selection state.

### AC-3 The palette communicates visible range within the full result count
Given the palette is showing a windowed subset of matches,
when it renders,
then Tau shows which results are visible out of the total match count.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, prompt history, transcript search/copy,
local persistence, transcript export, palette autocomplete/history, and alias
feedback slices,
when paging and full browsing land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Navigating beyond the first page keeps the selected command visible in
  the rendered palette. Maps to AC-1. Tier: Functional.
- C-02 `Home`/`End` and `PageUp`/`PageDown` move selection across the result
  set predictably. Maps to AC-2. Tier: Functional.
- C-03 Palette render shows the visible range and total match count while
  windowed. Maps to AC-3. Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  paging/full-browsing changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can reach commands beyond the first five visible suggestions
- Selection remains obvious while moving through large result sets
- Palette feedback reports the current visible window instead of only raw
  match count

## Key Decisions
- Selection remains index-based across the full match list
- Rendering computes a visible window around the current selection
- Paging stays keyboard-first and layered on existing palette behavior
