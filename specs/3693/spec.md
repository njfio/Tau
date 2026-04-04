# Spec: Issue #3693 - Add reverse placeholder cycling and active feedback to `tau-tui` command palette

Status: Implemented

## Problem Statement
Issue `#3692` made scaffolded command-palette placeholders directly editable,
but the operator flow is still one-directional and visually implicit. Once Tau
jumps into a placeholder, there is no reverse cycling path to move back to an
earlier placeholder, and the palette does not clearly show which placeholder is
currently active. A stronger REPL should let operators move both directions
through unresolved placeholders and keep the active placeholder obvious in the
palette UI.

## Scope
In scope:
- reverse placeholder cycling for scaffolded command-palette inputs
- explicit command-palette feedback that shows the active placeholder token
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3693/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- placeholder value validation beyond the existing unresolved guardrails
- mouse-based placeholder selection
- changes to runtime command execution semantics
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 Reverse placeholder cycling moves to the previous unresolved span
Given a scaffolded command contains multiple unresolved placeholders and one of
the later placeholders is active,
when the operator presses reverse-tab from the command palette,
then Tau moves placeholder focus back to the previous unresolved placeholder
instead of leaving focus on the current placeholder.

### AC-2 The command palette shows which placeholder is active
Given a scaffold placeholder is active for editing,
when the command palette renders,
then Tau surfaces explicit feedback naming the active placeholder and its
position within the unresolved placeholder sequence.

### AC-3 Earlier M335 REPL slices and #3690/#3692 placeholder behavior do not regress
Given the existing runtime control, history, persistence, transcript tooling,
palette browsing, scaffolding, fuzzy filtering, placeholder guardrails, and
placeholder-aware editing,
when reverse placeholder cycling and active feedback land,
then scoped `tau-tui` regression coverage still passes and unresolved
placeholders still block command execution.

## Conformance Cases
- C-01 Pressing reverse-tab while a later placeholder is active moves focus to
  the previous unresolved placeholder span. Maps to AC-1. Tier: Functional.
- C-02 The command palette render shows the active placeholder token and its
  ordinal position while a scaffold placeholder is focused. Maps to AC-2. Tier:
  Functional.
- C-03 Existing `#3690` and `#3692` placeholder guardrails and broader
  `interactive::app_gateway_tests` coverage still pass. Maps to AC-3. Tier:
  Regression.

## Success Metrics / Observable Signals
- Operators can move backward through scaffold placeholders without manually
  re-entering the scaffold
- The palette UI makes the active placeholder obvious during inline editing
- Prior placeholder guardrails and editing behaviors remain intact

## Key Decisions
- Treat `BackTab` as the reverse counterpart to `Tab` for placeholder focus
- Keep active-placeholder feedback inside the existing command-palette overlay
  rather than introducing a separate status surface
- Preserve the existing unresolved-placeholder submission guardrail from `#3690`
