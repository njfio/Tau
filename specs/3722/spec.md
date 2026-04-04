# Spec: Issue #3722 - Canonicalize slash-prefixed typed Enter preview guidance in tau-tui command palette

Status: Implemented

## Problem Statement
After `#3721`, slash-prefixed typed command-palette submissions with arguments
execute correctly, but the palette preview still renders the raw slash-prefixed
string instead of the actual Enter target. Inputs like `/search apples` and
`/copy-last assistant` therefore preview a command string that is slightly
different from what Tau will execute.

## Scope
In scope:
- Enter-preview guidance for slash-prefixed typed command-palette submissions
- canonical preview targets for argument-carrying slash-prefixed commands
- focused RED/GREEN gateway coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3722/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command execution changes
- filtering or ranking changes
- placeholder blocking semantics
- new commands or aliases

## Acceptance Criteria
### AC-1 Slash-prefixed typed search preview shows the canonical Enter target
Given the operator types `/search apples` directly into the command palette,
when Tau renders the Enter-preview guidance,
then it shows the canonical target `search apples` instead of the raw
slash-prefixed form.

### AC-2 Slash-prefixed typed clipboard preview shows the canonical Enter target
Given the operator types `/copy-last assistant` directly into the command
palette,
when Tau renders the Enter-preview guidance,
then it shows the canonical target `copy-last assistant` instead of the raw
slash-prefixed form.

### AC-3 Existing unprefixed typed preview guidance does not regress
Given earlier unprefixed typed-command preview guidance,
when slash-prefixed canonical preview guidance lands,
then unprefixed typed previews still render as before.

## Conformance Cases
- C-01 Typing `/search apples` in the command palette renders
  `Enter runs typed: search apples`. Maps to AC-1. Tier: Functional.
- C-02 Typing `/copy-last assistant` in the command palette renders
  `Enter runs typed: copy-last assistant`. Maps to AC-2. Tier: Functional.
- C-03 Existing unprefixed typed preview coverage still passes after preview
  canonicalization. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Preview guidance matches the actual Enter target for slash-prefixed typed
  palette submissions
- Operators no longer see slash-prefixed preview targets that differ from
  execution
- Earlier unprefixed typed-preview coverage remains green

## Key Decisions
- Canonicalize only the preview target string, not the stored history entry
- Reuse the same first-token slash normalization boundary established in `#3721`
