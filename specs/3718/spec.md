# Spec: Issue #3718 - Surface literal scaffold-token provenance in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview now surfaces alias, placeholder, section,
and summary provenance, but it still does not tell the operator when the
current query matched the selected command through a literal scaffold argument
token such as `assistant` in `copy-last [user|assistant|system|tool]`. That
leaves scaffold-driven discovery less transparent than the other preview
provenance flows.

## Scope
In scope:
- command-palette preview provenance for literal scaffold argument tokens
- exact scaffold-token preview feedback for the selected command
- suppressing scaffold-token provenance when the canonical command name was typed
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3718/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- command execution behavior changes
- new commands or scaffold text changes
- alias, placeholder, section, or summary provenance changes

## Acceptance Criteria
### AC-1 Literal scaffold token surfaces provenance in the preview
Given the current command-palette query matches the selected command through a
literal scaffold argument token,
when Tau renders the preview block,
then it shows which scaffold token matched the command.

### AC-2 Canonical query does not show scaffold-token provenance
Given the current command-palette query matches the selected command by its
canonical command name,
when Tau renders the preview block,
then it does not add scaffold-token provenance noise to the preview.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when scaffold-token provenance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `assistant` in the command palette shows
  `Matched via scaffold token: assistant` in the preview. Maps to AC-1.
  Tier: Functional.
- C-02 Typing `system` in the command palette shows
  `Matched via scaffold token: system` in the preview. Maps to AC-1.
  Tier: Functional.
- C-03 Typing `copy-last` in the command palette does not show any
  `Matched via scaffold token:` line in the preview. Maps to AC-2.
  Tier: Functional.
- C-04 Existing preview and navigation coverage still passes after
  scaffold-token provenance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell when a command was discovered via a literal scaffold token
- Scaffold-driven queries become as inspectable as alias, placeholder, section, and summary queries
- Preview stays quiet for canonical-name matches

## Key Decisions
- Keep scaffold-token provenance in the preview only; do not change matching behavior
- Show the exact scaffold token that matched
- Limit the slice to literal scaffold tokens rather than placeholder or multi-token provenance
