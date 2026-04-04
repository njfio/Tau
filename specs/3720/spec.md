# Spec: Issue #3720 - Reuse real Enter preview guidance in empty-match tau-tui command palette state

Status: Implemented

## Problem Statement
The `tau-tui` command palette already computes accurate Enter-preview guidance
for typed commands, including unresolved-placeholder blocking, but the empty-
match overlay still hardcodes `Enter runs the typed command as-is.`. That means
no-match explicit commands hide their real Enter target, and no-match inputs
with unresolved placeholders still get misleading execution text.

## Scope
In scope:
- no-match command-palette Enter-preview guidance
- exact typed-command Enter guidance in the no-match overlay
- unresolved-placeholder blocking guidance in the no-match overlay
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3720/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- matched-command preview guidance changes
- submit-time behavior changes
- new commands or scaffold changes

## Acceptance Criteria
### AC-1 No-match typed commands show their real Enter target
Given the command palette has no matching suggestions and the input is treated
as an explicit typed command with no unresolved placeholders,
when Tau renders the no-match overlay,
then it shows the real typed-command Enter target instead of generic fallback text.

### AC-2 No-match unresolved placeholders show blocking guidance
Given the command palette has no matching suggestions and the explicit typed
input still contains unresolved placeholders,
when Tau renders the no-match overlay,
then it shows Enter-blocked guidance that names the placeholder(s) to replace.

### AC-3 Existing no-match feedback does not regress
Given the earlier no-match overlay behavior,
when real Enter guidance lands there,
then `No matching commands` still renders and scoped `tau-tui` regression
coverage still passes.

## Conformance Cases
- C-01 Typing `resume mission-42` in the command palette shows
  `Enter runs typed: resume mission-42` while the palette still says
  `No matching commands`. Maps to AC-1. Tier: Functional.
- C-02 Typing `bogus <query>` in the command palette shows
  `Enter blocked: replace <query>` while the palette still says
  `No matching commands`. Maps to AC-2. Tier: Functional.
- C-03 Existing no-match overlay coverage still passes after the guidance swap.
  Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can see the actual Enter effect in no-match palette states
- No-match placeholder inputs stop claiming they will run as-is
- Existing no-match warning remains visible

## Key Decisions
- Reuse the existing Enter-preview helper instead of duplicating logic in the no-match branch
- Keep the change preview-only; do not alter matching or execution behavior
