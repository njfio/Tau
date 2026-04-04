# Spec: Issue #3690 - Add placeholder guardrails to `tau-tui` command palette scaffolds

Status: Implemented

## Problem Statement
The `tau-tui` command palette now scaffolds parameterized commands such as
`resume <mission-id>` and `search <query>`, but those templates can still be
submitted unchanged. When that happens, Tau tries to execute placeholder text as
real input, which is noisy and misleading. A stronger REPL palette should catch
unresolved placeholder tokens and convert them into clear operator guidance
instead of bogus command execution.

## Scope
In scope:
- detect unresolved scaffold placeholders in command palette submissions
- prevent execution when placeholder tokens are still present
- surface clear guidance about the missing argument(s)
- add focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3690/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- inline placeholder cursor-jump editing
- schema-driven argument validation beyond unresolved placeholders
- dynamic command registration
- gateway/runtime protocol changes

## Acceptance Criteria
### AC-1 Scaffold placeholders do not execute as real command arguments
Given the operator autocompletes a scaffolded command and leaves placeholder
tokens unchanged,
when they submit the command palette,
then Tau does not execute the command against those placeholder values.

### AC-2 Tau surfaces a clear operator hint for unresolved placeholders
Given a palette submission still contains placeholder tokens,
when Tau blocks execution,
then it surfaces a readable hint explaining which placeholder(s) must be
replaced before running the command.

### AC-3 Commands with real argument values still execute normally
Given the operator replaces the scaffold placeholders with real values,
when they submit the command palette,
then Tau executes the command on the normal command path.

### AC-4 Earlier M335 REPL slices do not regress
Given the existing runtime control, history, persistence, transcript export,
palette autocomplete/history, aliases, paging, previews, grouping, scaffolding,
and fuzzy filtering slices,
when placeholder guardrails land,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Submitting a raw scaffold template blocks execution. Maps to AC-1. Tier:
  Functional.
- C-02 Blocked scaffold submission surfaces a placeholder guidance message.
  Maps to AC-2. Tier: Functional.
- C-03 Replacing placeholders with real values still executes the command.
  Maps to AC-3. Tier: Functional.
- C-04 Existing M335 `interactive::app_gateway_tests` continue to pass after
  placeholder-guardrail changes. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Placeholder templates no longer trigger bogus runtime work
- Operators see direct guidance about what needs to be filled in
- Real command submissions remain unchanged

## Key Decisions
- Placeholder detection is token-based and bounded to scaffold markers like
  `<argument>`
- Blocked scaffold submissions should preserve the operator’s context and not
  silently discard the command
- Real-valued command submissions stay on the existing execution path
