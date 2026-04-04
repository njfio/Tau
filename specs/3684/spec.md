# Spec: Issue #3684 - Add command palette aliases and richer feedback to `tau-tui` REPL

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports suggestions, autocomplete, and
history, but it still behaves like a thin matcher. Operators still need to
memorize full command names, and the palette does not clearly explain what is
selected, how many matches exist, or when the current query has no matches.
A stronger REPL launcher should support short aliases and provide immediate,
readable feedback while the operator types.

## Scope
In scope:
- add bounded alias support for the command palette command catalog
- match aliases during palette filtering and execute aliases correctly
- surface richer palette feedback for match counts, active selection, and
  no-match states
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3684/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- fuzzy typo correction
- dynamic command registration
- mouse-based palette selection
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 Aliases are discoverable in the command palette
Given the operator opens the command palette,
when they inspect matching commands,
then Tau renders each command with any configured aliases and short feedback
about current match state.

### AC-2 Alias input executes the intended canonical command
Given the operator types a supported alias into the command palette,
when they press `Enter`,
then Tau resolves the alias to the canonical command and executes the intended
behavior.

### AC-3 The palette communicates no-match and selection state clearly
Given the operator types a query that narrows or eliminates matches,
when the palette renders,
then Tau shows readable feedback for the active selection and for zero-match
states instead of a silent empty list.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, prompt history, transcript search/copy,
local persistence, transcript export, and palette autocomplete/history slices,
when alias and feedback improvements land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Palette render includes aliases and match-state feedback. Maps to AC-1.
  Tier: Functional.
- C-02 Typing a supported alias and pressing `Enter` executes the canonical
  command. Maps to AC-2. Tier: Functional.
- C-03 Palette render surfaces a readable zero-match state. Maps to AC-3.
  Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  the alias/feedback changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can launch common commands with shorter keystrokes
- Palette render exposes match counts and active selection context
- Zero-match queries are obvious instead of looking broken

## Key Decisions
- Aliases stay static and explicit in the command catalog
- The palette executes canonical commands while still accepting alias input
- Feedback stays inside the existing palette overlay instead of introducing a
  new panel
