# Spec: Issue #3717 - Surface summary-match provenance in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview now surfaces alias, scaffold placeholder,
and section provenance, but it still does not tell the operator when the
current query matched the selected command through its summary text. That
leaves summary-driven palette discovery less transparent than the other
preview-provenance flows.

## Scope
In scope:
- command-palette preview summary-match provenance
- exact summary-query preview feedback for the selected command
- suppressing summary provenance when the canonical command name was typed
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3717/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- command execution behavior changes
- new commands or summary text changes
- alias, placeholder, or section provenance changes

## Acceptance Criteria
### AC-1 Summary query surfaces summary-match provenance in the preview
Given the current command-palette query matches the selected command through its
summary text,
when Tau renders the preview block,
then it shows which summary token matched the command.

### AC-2 Canonical query does not show summary-match provenance
Given the current command-palette query matches the selected command by its
canonical command name,
when Tau renders the preview block,
then it does not add summary-match provenance noise to the preview.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when summary-match provenance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `mutating` in the command palette shows
  `Matched via summary: mutating` in the preview. Maps to AC-1.
  Tier: Functional.
- C-02 Typing `checkpointed` in the command palette shows
  `Matched via summary: checkpointed` in the preview. Maps to AC-1.
  Tier: Functional.
- C-03 Typing `copy-target` in the command palette does not show any
  `Matched via summary:` line in the preview. Maps to AC-2.
  Tier: Functional.
- C-04 Existing preview and navigation coverage still passes after
  summary-match provenance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell when a command was discovered via its summary text
- Summary-driven queries become as inspectable as alias, placeholder, and section queries
- Preview stays quiet for canonical-name matches

## Key Decisions
- Keep summary provenance in the preview only; do not change matching behavior
- Show the exact summary token that matched
- Limit the slice to exact summary-token provenance rather than broader phrase provenance
