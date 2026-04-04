# Spec: Issue #3715 - Surface scaffold placeholder-match provenance in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview now shows the selected command, section,
scaffold, placeholders, Enter behavior, and alias provenance, but it still does
not tell the operator when the current query matched a scaffold placeholder
token like `<mission-id>` or `<query>` rather than the command name. That
leaves placeholder-driven palette discovery less transparent than canonical
name or alias-driven queries.

## Scope
In scope:
- command-palette preview placeholder-match provenance
- exact query-to-placeholder preview feedback for the selected command
- suppressing placeholder provenance when the canonical command name was typed
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3715/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- command execution behavior changes
- new commands or placeholder syntax
- alias or section provenance changes

## Acceptance Criteria
### AC-1 Placeholder query surfaces placeholder-match provenance in the preview
Given the current command-palette query matches one of the selected command's
scaffold placeholders,
when Tau renders the preview block,
then it shows which placeholder token matched the command.

### AC-2 Canonical query does not show placeholder-match provenance
Given the current command-palette query matches the selected command by its
canonical command name,
when Tau renders the preview block,
then it does not add placeholder-match provenance noise to the preview.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when placeholder-match provenance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `mission-id` in the command palette shows
  `Matched via placeholder: <mission-id>` in the preview. Maps to AC-1.
  Tier: Functional.
- C-02 Typing `query` in the command palette shows
  `Matched via placeholder: <query>` in the preview. Maps to AC-1.
  Tier: Functional.
- C-03 Typing `mission` in the command palette does not show any
  `Matched via placeholder:` line in the preview. Maps to AC-2.
  Tier: Functional.
- C-04 Existing preview and navigation coverage still passes after
  placeholder-match provenance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell when a command was discovered via scaffold placeholder text
- Placeholder-driven queries become as inspectable as alias-driven queries
- Preview stays quiet for canonical-name matches

## Key Decisions
- Keep placeholder provenance in the preview only; do not change matching behavior
- Show the exact placeholder token that matched
- Normalize the typed query against placeholder text without requiring angle brackets
