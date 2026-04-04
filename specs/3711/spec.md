# Spec: Issue #3711 - Surface section and scaffold details in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview already shows the selected command name,
usage, aliases, and summary, but it omits two details operators need while
deciding what to run: the command's section and the exact scaffold template
that `Tab` will insert. Section headers remain visible in the suggestion list,
yet that context disappears once the operator looks at the selected preview.
Parameterized commands also lack an explicit preview of the scaffold string that
drives placeholder-based execution.

## Scope
In scope:
- command palette preview section detail
- command palette preview scaffold detail
- preview updates for changed selections
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3711/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering changes
- new command-palette commands
- runtime/gateway changes
- scaffold execution behavior

## Acceptance Criteria
### AC-1 Preview surfaces the selected command section
Given the command palette is open with a selected command,
when Tau renders the preview block,
then it shows that command's section label alongside the existing preview
details.

### AC-2 Preview surfaces the exact scaffold template for parameterized commands
Given the operator changes selection to a command with scaffold placeholders,
when Tau renders the preview block,
then it shows the exact scaffold string that `Tab` autocomplete will insert.

### AC-3 Existing command-palette preview behavior does not regress
Given the earlier selected-command preview behavior and selection-tracking
coverage,
when section and scaffold preview details land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Opening the command palette shows `Section: Session` and
  `Scaffold: quit` in the selected preview for `quit`. Maps to AC-1.
  Tier: Functional.
- C-02 Moving selection to `resume` shows `Section: Missions` and
  `Scaffold: resume <mission-id>` in the selected preview. Maps to AC-2.
  Tier: Functional.
- C-03 Existing preview and selection-tracking coverage still passes after the
  richer preview lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can see the current command category directly in the preview block
- Operators can confirm the exact scaffold string before autocompleting it
- Preview remains aligned with the current selection across navigation

## Key Decisions
- Treat section and scaffold as additive preview details, not replacements for
  usage or summary
- Keep the slice in the overlay render path only
- Preserve the current command-palette selection and navigation behavior
