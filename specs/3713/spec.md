# Spec: Issue #3713 - Surface Enter execution target in tau-tui command palette preview

Status: Implemented

## Problem Statement
The `tau-tui` command palette preview still uses a generic `Enter run` hint
when matches exist, even though `Enter` can mean either "run the selected
command token" or "run the exact typed command text." The no-match state
already tells operators that `Enter` runs the typed command as-is, but the
matching state never clarifies what will actually execute. That makes the
preview less trustworthy at the moment the operator is deciding whether to send
typed input, accept a selected command, or autocomplete a scaffold first.

## Scope
In scope:
- command palette preview Enter-action guidance
- differentiating selected-command execution from typed-command execution
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3713/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering changes
- new command-palette commands
- runtime/gateway changes
- command execution semantics

## Acceptance Criteria
### AC-1 Matching preview surfaces selected-command execution guidance
Given the command palette has matches and the current input resolves to the
selected command,
when Tau renders the preview block,
then it shows which selected command `Enter` will run.

### AC-2 Matching preview surfaces typed-command execution guidance for explicit input
Given the current command-palette input is an explicit command string that will
execute as typed,
when Tau renders the preview block,
then it shows that `Enter` will run the typed command text instead of only
showing a generic run hint.

### AC-3 Existing preview behavior does not regress
Given the earlier command-palette preview and navigation behavior,
when Enter-action guidance lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Typing `hel` in the command palette shows `Enter runs selected: help`
  in the preview because the current input resolves to the selected command.
  Maps to AC-1. Tier: Functional.
- C-02 Typing `copy-last assistant` in the command palette shows
  `Enter runs typed: copy-last assistant` in the preview. Maps to AC-2.
  Tier: Functional.
- C-03 Existing preview and navigation coverage still passes after Enter-action
  guidance lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can tell what `Enter` will execute before pressing it
- Explicit typed commands no longer look ambiguous when matches exist
- Preview remains aligned with the selected command and typed input state

## Key Decisions
- Add execution guidance to the existing preview instead of changing command
  resolution behavior
- Reflect current submission semantics exactly rather than inventing a new mode
- Keep the slice in the command-palette preview path only
