# Spec: Issue #3687 - Group `tau-tui` command palette commands into operator sections

Status: Implemented

## Problem Statement
The `tau-tui` command palette now has autocomplete, aliases, paging, and
selected-command previews, but the command list is still a flat sequence.
Browsing a flat list makes the launcher feel mechanical and forces operators to
scan unrelated commands together. A stronger REPL palette should group commands
into operator-oriented sections so browsing conveys intent as well as command
names.

## Scope
In scope:
- add section/category metadata to the command palette catalog
- render section labels inside the paged command palette
- preserve selection, paging, and preview behavior with grouped rendering
- add focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3687/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- dynamic command registration
- section collapse/expand interactions
- mouse-based browsing
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 The command palette renders operator-oriented sections
Given the operator opens the command palette,
when commands are displayed,
then Tau renders readable section labels around related commands rather than a
single undifferentiated list.

### AC-2 Grouped rendering remains correct while selecting and paging
Given the operator moves through command palette results,
when the active selection changes or the visible page changes,
then Tau keeps selection, visible range, and selected-command preview correct
within the grouped palette render.

### AC-3 Filtered queries preserve useful grouping behavior
Given the operator types a query that narrows the match set,
when the palette renders those filtered results,
then Tau shows only the relevant section labels instead of stale or unrelated
group headings.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, history, persistence, transcript export,
palette autocomplete/history, aliases, paging, and preview-detail slices,
when grouped command sections land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Palette render includes section labels for grouped commands. Maps to
  AC-1. Tier: Functional.
- C-02 Paged browsing preserves selection and preview under grouped rendering.
  Maps to AC-2. Tier: Functional.
- C-03 Filtered results only show relevant section labels. Maps to AC-3.
  Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  grouped-render changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can browse commands by intent instead of memorizing a flat list
- Section labels remain meaningful under filtering and paging
- Preview and selection stay stable under grouped rendering

## Key Decisions
- Sections are static metadata on the command catalog
- Group labels render inline in the existing palette overlay
- Grouping enhances browsing without changing command execution semantics
