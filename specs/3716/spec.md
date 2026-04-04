# Spec: Issue #3716 - Surface section-match provenance in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview now surfaces alias and scaffold
placeholder provenance, but it still does not tell the operator when the
current query matched the selected command through its section label like
`Clipboard` or `Runtime`. That leaves section-driven palette discovery less
transparent than other preview-provenance flows.

## Scope
In scope:
- command-palette preview section-match provenance
- exact section-query preview feedback for the selected command
- suppressing section provenance when the canonical command name was typed
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3716/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- command execution behavior changes
- new commands or sections
- alias or placeholder provenance changes

## Acceptance Criteria
### AC-1 Section query surfaces section-match provenance in the preview
Given the current command-palette query matches the selected command through its
section label,
when Tau renders the preview block,
then it shows which section label matched the command.

### AC-2 Canonical query does not show section-match provenance
Given the current command-palette query matches the selected command by its
canonical command name,
when Tau renders the preview block,
then it does not add section-match provenance noise to the preview.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when section-match provenance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `clipboard` in the command palette shows
  `Matched via section: Clipboard` in the preview. Maps to AC-1.
  Tier: Functional.
- C-02 Typing `runtime` in the command palette shows
  `Matched via section: Runtime` in the preview. Maps to AC-1.
  Tier: Functional.
- C-03 Typing `status` in the command palette does not show any
  `Matched via section:` line in the preview. Maps to AC-2.
  Tier: Functional.
- C-04 Existing preview and navigation coverage still passes after
  section-match provenance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell when a command was discovered via a section label
- Section-driven queries become as inspectable as alias and placeholder queries
- Preview stays quiet for canonical-name matches

## Key Decisions
- Keep section provenance in the preview only; do not change matching behavior
- Show the exact section label that matched
- Limit the slice to exact section-label provenance rather than broader metadata provenance
