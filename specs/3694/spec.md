# Spec: Issue #3694 - Auto-focus first placeholder when command-palette scaffolds a parameterized command

Status: Implemented

## Problem Statement
Issue `#3688` taught the command palette to scaffold parameterized commands
like `resume <mission-id>`, and issues `#3692` and `#3693` made those
placeholders editable and navigable. The common autocomplete path still costs
an unnecessary extra keystroke, though: after `Tab` inserts a parameterized
scaffold, Tau leaves the cursor at the end of the command instead of dropping
the operator directly into the first placeholder. A stronger REPL should make
the first autocomplete keystroke both scaffold and focus the first unresolved
placeholder when the selected command is parameterized.

## Scope
In scope:
- parameterized command scaffolds auto-focus their first unresolved placeholder
  on autocomplete
- alias-driven autocomplete keeps using the canonical scaffold and also focuses
  the first unresolved placeholder
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3694/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- multi-placeholder validation changes
- runtime command execution changes
- gateway/runtime protocol changes
- changes to non-parameterized command autocomplete semantics

## Acceptance Criteria
### AC-1 Canonical autocomplete focuses the first placeholder for parameterized commands
Given the operator selects a parameterized command such as `resume`,
when they trigger autocomplete from the command palette,
then Tau inserts the scaffold and focuses the first unresolved placeholder so
the next typed characters replace that placeholder immediately.

### AC-2 Alias-driven autocomplete focuses the first placeholder on the canonical scaffold
Given the operator selects a parameterized command through an alias such as
`rs` for `resume`,
when they trigger autocomplete from the command palette,
then Tau inserts the canonical scaffold and focuses its first unresolved
placeholder so the next typed characters replace that placeholder immediately.

### AC-3 Earlier M335 scaffolding and placeholder behavior do not regress
Given the existing autocomplete scaffolding, placeholder editing, reverse
cycling, guardrails, and broader M335 REPL behavior,
when auto-focus lands for parameterized scaffolds,
then scoped `tau-tui` regression coverage still passes and non-parameterized
autocomplete behavior remains intact.

## Conformance Cases
- C-01 Autocomplete on a parameterized command inserts its scaffold and routes
  the next typed value into the first placeholder without requiring a second
  `Tab`. Maps to AC-1. Tier: Functional.
- C-02 Alias-selected parameterized commands autocomplete to the canonical
  scaffold and route the next typed value into the first placeholder without
  requiring a second `Tab`. Maps to AC-2. Tier: Functional.
- C-03 Existing non-parameterized autocomplete, placeholder guardrails, and
  broader `interactive::app_gateway_tests` coverage still pass. Maps to AC-3.
  Tier: Regression.

## Success Metrics / Observable Signals
- Operators can complete common single-placeholder commands with one less
  keystroke after autocomplete
- Alias-driven parameterized autocomplete keeps the canonical scaffold path
- Existing simple-command autocomplete and placeholder guardrails remain stable

## Key Decisions
- Keep `Tab` as the single autocomplete trigger and make parameterized
  scaffolds auto-enter their first placeholder immediately
- Preserve existing non-parameterized autocomplete behavior without adding
  synthetic placeholder state where none exists
- Reuse the existing placeholder-span machinery from `#3692` and `#3693`
