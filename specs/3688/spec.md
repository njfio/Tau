# Spec: Issue #3688 - Add inline argument scaffolding to `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now supports autocomplete, aliases, paging,
selected-command previews, and grouped sections, but operators still have to
manually type argument skeletons for parameterized commands like `resume`,
`mission`, `search`, and `save-transcript`. A stronger REPL launcher should
turn selection into action by scaffolding the right command template directly
into the palette input.

## Scope
In scope:
- add explicit scaffold templates for parameterized palette commands
- make palette autocomplete insert command templates where appropriate
- keep aliases, selection, paging, grouping, and preview behavior aligned with
  scaffolding
- add focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3688/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- placeholder cursor jumping within arguments
- inline validation of argument values
- dynamic command registration
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 Palette autocomplete scaffolds argument templates for parameterized commands
Given the operator selects a parameterized command in the palette,
when they trigger autocomplete,
then Tau inserts a useful command template with argument placeholders instead of
only the bare command name.

### AC-2 Alias-driven selection scaffolds the canonical template
Given the operator selects a command via alias matching,
when they trigger autocomplete,
then Tau inserts the canonical command template rather than the alias token.

### AC-3 Non-parameterized commands keep simple autocomplete behavior
Given the operator selects a command that takes no arguments,
when they trigger autocomplete,
then Tau still inserts the simple command name without regressing prior palette
behavior.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, history, persistence, transcript export,
palette autocomplete/history, aliases, paging, previews, and grouping slices,
when inline argument scaffolding lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Autocomplete on a parameterized command inserts its scaffold template.
  Maps to AC-1. Tier: Functional.
- C-02 Alias-selected commands autocomplete to canonical templates. Maps to
  AC-2. Tier: Functional.
- C-03 Non-parameterized commands still autocomplete to simple names. Maps to
  AC-3. Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  scaffolding changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can populate command templates with fewer keystrokes
- Alias use still leads to canonical command scaffolds
- Bare commands without arguments retain the old fast-path behavior

## Key Decisions
- Scaffold templates are explicit metadata on the command catalog
- Autocomplete remains the trigger for inserting templates
- Canonical command templates are preferred over alias-shaped input
