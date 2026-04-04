# Spec: Issue #3714 - Surface alias-match provenance in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview now shows the selected command, scaffold,
placeholders, and Enter behavior, but it still does not tell the operator when
the current query matched through an alias token instead of the canonical
command name. For short alias-driven flows like `q` or `rs`, the preview shows
the alias list but not the specific alias that caused the current selection.
That makes alias-based command discovery less transparent than canonical-name
searches.

## Scope
In scope:
- command-palette preview alias-match provenance
- exact alias-query preview feedback for the selected command
- suppressing alias provenance when the canonical command name was typed
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3714/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- command execution behavior changes
- new commands or aliases
- placeholder or Enter preview changes

## Acceptance Criteria
### AC-1 Alias query surfaces alias-match provenance in the preview
Given the current command-palette query exactly matches one of the selected
command's aliases,
when Tau renders the preview block,
then it shows which alias token matched the command.

### AC-2 Canonical query does not show alias-match provenance
Given the current command-palette query matches the selected command by its
canonical command name,
when Tau renders the preview block,
then it does not add alias-match provenance noise to the preview.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when alias-match provenance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `q` in the command palette shows `Matched via alias: q` in the
  preview while `quit` is selected. Maps to AC-1. Tier: Functional.
- C-02 Typing `rs` in the command palette shows `Matched via alias: rs` in the
  preview while `resume` is selected. Maps to AC-1. Tier: Functional.
- C-03 Typing `quit` in the command palette does not show any `Matched via alias:`
  line in the preview. Maps to AC-2. Tier: Functional.
- C-04 Existing preview and navigation coverage still passes after alias-match
  provenance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell when the palette selected a command via an alias rather
  than the canonical name
- Alias-driven queries become as inspectable as canonical command queries
- The preview stays quiet for canonical-name matches

## Key Decisions
- Keep alias provenance in the preview only; do not change matching behavior
- Show the exact alias token that matched, not just a generic alias note
- Suppress provenance when the typed query matched the command name directly
