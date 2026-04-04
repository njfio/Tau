# Spec: Issue #3707 - Match `tau-tui` command palette queries against usage and scaffold text

Status: Implemented

## Problem Statement
The `tau-tui` command palette already matches queries against command names,
aliases, summaries, and fuzzy subsequences, but it ignores the command `usage`
and scaffold text that operators actually see in previews and scaffolds. That
means queries like `mission-id` or `assistant` fail to surface the commands that
accept those arguments even though the palette already exposes that text as part
of the command model. This makes command discovery weaker than the rest of the
operator REPL.

## Scope
In scope:
- usage-text matching in command-palette filtering
- scaffold/placeholder token matching in command-palette filtering
- preserving existing exact/prefix/fuzzy behavior for names and aliases
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3707/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- new command-palette commands
- runtime/gateway changes
- broader command ranking changes beyond adding new match sources
- transcript search behavior

## Acceptance Criteria
### AC-1 Usage-text queries surface matching commands
Given a command's `usage` text includes argument tokens that do not appear in
its name, alias, or summary,
when the operator queries those usage tokens,
then Tau surfaces that command in command-palette matches.

### AC-2 Scaffold placeholder tokens surface matching commands
Given a command scaffold includes placeholder tokens like `<mission-id>`,
when the operator queries those placeholder tokens,
then Tau surfaces commands whose scaffolds include them.

### AC-3 Existing command-palette filtering behavior does not regress
Given the earlier exact/prefix/fuzzy command-palette matching behavior,
when usage/scaffold matching lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Querying `assistant` surfaces `copy-last` because its usage includes
  `[user|assistant|system|tool]`. Maps to AC-1. Tier: Functional.
- C-02 Querying `mission-id` surfaces `mission` and `resume` because their
  scaffolds/usages include `<mission-id>`. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette filter and preview coverage still passes after
  the new match sources land. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can discover commands by remembered argument names and placeholder
  tokens instead of only by command names
- Palette filtering feels aligned with the usage/scaffold text already shown in
  previews and autocomplete
- Existing command matching remains stable

## Key Decisions
- Treat `usage` and scaffold text as additional match sources, not a separate
  search mode
- Preserve the existing exact/prefix/fuzzy ranking structure
- Keep the slice limited to filtering behavior rather than UI redesign
