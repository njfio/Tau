# Spec: Issue #3709 - Match multi-token `tau-tui` command palette queries across command metadata

Status: Implemented

## Problem Statement
The `tau-tui` command palette now matches separator-normalized names, usage, and
scaffold text, but multi-token queries still need to appear contiguously inside
one normalized field. Queries like `copy assistant` or `save path` do not
surface commands whose meaning is obvious when you combine their command name
with usage metadata, because the current matcher only checks one field at a
time. This leaves command discovery weaker than the rest of the operator REPL.

## Scope
In scope:
- multi-token matching across command metadata fields
- combined matching over command name, summary, usage, scaffold, and aliases
- preserving existing exact/prefix/fuzzy and separator-normalized behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3709/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- runtime/gateway changes
- new command-palette commands
- broader ranking redesign beyond multi-token metadata matching
- transcript search behavior

## Acceptance Criteria
### AC-1 Multi-token queries can match across command name and usage text
Given a command’s name and usage each contribute different parts of the query,
when the operator searches using those combined tokens,
then Tau surfaces the command in command-palette matches.

### AC-2 Multi-token queries can match across command name and argument metadata
Given a command name contributes one token and its usage/scaffold contributes
another,
when the operator searches using both tokens together,
then Tau surfaces the command even if no single field contains the full query
contiguously.

### AC-3 Existing command-palette filtering behavior does not regress
Given the earlier exact/prefix/fuzzy, usage/scaffold-aware, and
separator-normalized matching behavior,
when multi-token metadata matching lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 Querying `copy assistant` surfaces `copy-last` because `copy` comes from
  the command name and `assistant` comes from usage metadata. Maps to AC-1.
  Tier: Functional.
- C-02 Querying `save path` surfaces `save-transcript` because `save` comes from
  the command name and `path` comes from usage metadata. Maps to AC-2. Tier:
  Functional.
- C-03 Existing command-palette filter and submission coverage still passes
  after multi-token metadata matching lands. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can discover commands using natural multi-word queries instead of
  remembering exact contiguous substrings
- Matching feels aligned with the full command metadata already shown in
  previews and scaffolds
- Existing palette ranking remains stable

## Key Decisions
- Treat multi-token matching as a metadata-level filter enhancement rather than
  a new search mode
- Preserve the existing ranking tiers and broaden only what counts as a match
- Limit the slice to metadata matching instead of UI or execution changes
