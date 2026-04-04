# Spec: Issue #3712 - Surface scaffold placeholder summary in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview now shows command section and scaffold
details, but it still does not summarize unresolved scaffold placeholders
before autocomplete happens. Operators can see a parameterized scaffold like
`resume <mission-id>`, yet the preview gives no direct placeholder summary until
placeholder mode is already active. That leaves the preview one step behind the
operator workflow.

## Scope
In scope:
- command palette preview placeholder summary for parameterized commands
- keeping simple-command previews free of placeholder noise
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3712/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering changes
- new command-palette commands
- runtime/gateway changes
- active placeholder editing behavior

## Acceptance Criteria
### AC-1 Parameterized command previews surface placeholder summary
Given the selected command scaffold contains placeholder tokens,
when Tau renders the command-palette preview,
then it shows a placeholder summary derived from that scaffold.

### AC-2 Simple command previews stay free of unnecessary placeholder detail
Given the selected command scaffold contains no placeholders,
when Tau renders the command-palette preview,
then it does not show a placeholder summary line.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when placeholder summaries land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Moving selection to `resume` shows `Placeholders: <mission-id>` in the
  selected preview. Maps to AC-1. Tier: Functional.
- C-02 Opening the command palette on the default `quit` selection does not
  render a `Placeholders:` preview line. Maps to AC-2. Tier: Functional.
- C-03 Existing preview and navigation coverage still passes after placeholder
  summary support lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can see unresolved scaffold requirements before entering
  placeholder-edit mode
- Simple commands remain visually lean in the preview pane
- Preview remains aligned with the selected command across navigation

## Key Decisions
- Show placeholder summary only for commands that actually have placeholders
- Derive the summary from the scaffold string instead of duplicating metadata
- Keep the slice in the command-palette preview path only
