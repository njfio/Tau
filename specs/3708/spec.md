# Spec: Issue #3708 - Normalize separator punctuation in `tau-tui` command palette queries

Status: Implemented

## Problem Statement
The `tau-tui` command palette now matches names, aliases, summaries, usage, and
scaffold text, but it still treats separator punctuation literally. Queries like
`copy last` or `mission id` do not match hyphenated command names and scaffold
placeholders such as `copy-last` and `<mission-id>`, even though those are the
natural tokens an operator is likely to remember. This keeps command discovery
slightly brittle and inconsistent with the rest of the REPL’s forgiving input
ergonomics.

## Scope
In scope:
- separator normalization across command-palette query matching
- space/hyphen-insensitive matching for command names, usage, and scaffold text
- preserving existing exact/prefix/fuzzy ranking behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3708/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- runtime/gateway changes
- new command-palette commands
- broader search ranking redesign
- transcript or chat search behavior

## Acceptance Criteria
### AC-1 Space-separated queries can match hyphenated command names
Given a command name contains hyphen separators,
when the operator queries the same tokens separated by spaces,
then Tau surfaces the command in command-palette matches.

### AC-2 Space-separated queries can match hyphenated scaffold placeholders
Given a scaffold or usage string contains a hyphenated placeholder token like
`mission-id`,
when the operator queries the same token with spaces,
then Tau surfaces commands whose usage or scaffold text contains that
placeholder.

### AC-3 Existing command-palette filtering behavior does not regress
Given the earlier exact/prefix/fuzzy and usage/scaffold-aware matching
behavior,
when separator normalization lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Querying `copy last` surfaces `copy-last` despite the hyphen in the
  command name. Maps to AC-1. Tier: Functional.
- C-02 Querying `mission id` surfaces `mission` and `resume` despite the hyphen
  in `<mission-id>`. Maps to AC-2. Tier: Functional.
- C-03 Existing command-palette filter and submission coverage still passes
  after separator normalization lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can discover hyphenated commands and placeholders using natural
  space-separated queries
- Query punctuation feels less brittle without changing the visible command
  catalog
- Existing palette ranking remains stable

## Key Decisions
- Normalize separator punctuation as part of matching, not as a separate search
  mode
- Keep the existing ranking tiers and only broaden what counts as a match
- Limit the slice to query normalization rather than broader fuzzy-search work
