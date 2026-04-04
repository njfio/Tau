# Spec: Issue #3686 - Add selected-command preview details to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports autocomplete, aliases, feedback, and
paging, but it still behaves like a thin launcher. Operators can find commands,
but they still have to remember command arguments and intended usage from
memory. A stronger REPL palette should preview the selected command so the
operator can confirm what it does and how to invoke it before executing it.

## Scope
In scope:
- extend the command catalog with bounded usage/help metadata
- render selected-command preview details inside the command palette
- keep the preview aligned with the current selection, aliases, and paging
- add focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3686/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- multi-line docs browser behavior
- dynamic command registration
- mouse interactions
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 The palette previews the selected command
Given the operator is in the command palette,
when a command is selected,
then Tau renders a compact preview for that command with usage-oriented details.

### AC-2 The preview tracks selection changes
Given the operator moves through command palette results,
when the active selection changes,
then the preview updates to reflect the newly selected command.

### AC-3 The preview remains useful with empty query and paged results
Given the operator opens the palette with no query or browses across pages,
when the palette renders,
then the selected-command preview still reflects the current selection without
breaking result navigation.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, history, persistence, transcript export,
palette autocomplete/history, aliases, and paging slices,
when selected-command preview details land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Palette render includes usage-oriented preview details for the selected
  command. Maps to AC-1. Tier: Functional.
- C-02 Changing the selected command updates the preview content. Maps to AC-2.
  Tier: Functional.
- C-03 Preview details remain correct while browsing paged results. Maps to AC-3.
  Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  preview-detail changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can confirm command semantics without leaving the palette
- The active command’s usage is visible before execution
- Paging and selection remain stable while preview content updates

## Key Decisions
- Preview details stay compact and live inside the existing overlay
- Command metadata is static and explicitly authored in the command catalog
- Preview follows the currently selected command, not the raw query text alone
