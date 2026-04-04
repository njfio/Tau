# Spec: Issue #3692 - Add placeholder-aware editing to `tau-tui` command palette scaffolds

Status: Implemented

## Problem Statement
The `tau-tui` command palette now scaffolds parameterized commands such as
`resume <mission-id>` and `search <query>`, and issue `#3690` blocks operators
from submitting those placeholders unchanged. That guardrail prevents bogus
execution, but completing scaffolded commands is still clumsy because the input
cursor remains at the end of the line and placeholder text must be removed one
character at a time. A stronger REPL should let operators jump into scaffolded
placeholders and replace them directly.

## Scope
In scope:
- placeholder-aware editing for scaffolded command-palette commands
- keyboard navigation that jumps into unresolved `<...>` placeholder spans
- replacement and clearing behavior that acts on the active placeholder span
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3692/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- multi-placeholder schema validation beyond the existing guardrails
- palette mouse editing
- changes to runtime command execution semantics
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 Tab moves focus into scaffold placeholders
Given the operator autocompletes a scaffolded command such as
`resume <mission-id>`,
when they press `Tab` again from the command palette,
then Tau moves editing focus into the unresolved placeholder instead of leaving
the cursor at the end of the scaffold string.

### AC-2 Typing replaces the active placeholder span
Given a scaffold placeholder is active for editing,
when the operator types replacement text,
then Tau replaces the whole placeholder token with the typed value instead of
appending after the placeholder token.

### AC-3 Backspace clears the active placeholder span as a unit
Given a scaffold placeholder is active for editing,
when the operator presses `Backspace`,
then Tau clears the placeholder token in one step so the operator can type a
real value immediately.

### AC-4 Earlier M335 REPL slices and #3690 guardrails do not regress
Given the existing runtime control, history, persistence, transcript tooling,
palette browsing, previews, scaffolding, fuzzy filtering, and placeholder
guardrails,
when placeholder-aware editing lands,
then scoped `tau-tui` regression coverage still passes and unresolved
placeholders still block command execution.

## Conformance Cases
- C-01 Pressing `Tab` on a scaffolded command moves editing into the unresolved
  placeholder span. Maps to AC-1. Tier: Functional.
- C-02 Typing after placeholder focus replaces the placeholder token instead of
  appending after it. Maps to AC-2. Tier: Functional.
- C-03 Pressing `Backspace` while a placeholder is active clears the whole
  placeholder span. Maps to AC-3. Tier: Functional.
- C-04 Existing `#3690` placeholder guardrail behavior and broader
  `interactive::app_gateway_tests` coverage still pass. Maps to AC-4. Tier:
  Regression.

## Success Metrics / Observable Signals
- Scaffolded commands become directly editable after autocomplete
- Operators no longer need to manually delete placeholder text one character at
  a time
- Unresolved placeholders still remain guarded from execution

## Key Decisions
- Reuse `Tab` as the placeholder-jump key after scaffold insertion
- Placeholder editing should stay bounded to unresolved `<...>` spans rather
  than introducing a general command-line parser
- Clearing a focused placeholder should preserve the surrounding command prefix
  and spacing
