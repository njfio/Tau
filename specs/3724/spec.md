# Spec: Issue #3724 - Canonicalize slash-prefixed alias typed Enter guidance in tau-tui command palette

Status: Implemented

## Problem Statement
After `#3721` through `#3723`, slash-prefixed typed command-palette inputs with
arguments execute canonically and keep their command match visible. But typed
alias inputs like `/rs mission-42` still preview the alias token instead of the
resolved command that Enter will actually run. That makes the palette's Enter
guidance less honest than the execution path.

## Scope
In scope:
- slash-prefixed alias typed Enter-preview guidance when arguments are present
- canonical command-name surfacing for typed alias submissions in the palette
- focused RED/GREEN gateway coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3724/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command execution changes
- command matching changes
- non-slash alias preview behavior
- alias-match provenance changes

## Acceptance Criteria
### AC-1 Slash-prefixed typed resume alias preview resolves to the canonical command
Given the operator types `/rs mission-42` into the command palette,
when Tau renders the Enter preview guidance,
then it says `Enter runs typed: resume mission-42` instead of echoing the alias
token.

### AC-2 Slash-prefixed typed mission alias preview resolves to the canonical command
Given the operator types `/mi mission-42` into the command palette,
when Tau renders the Enter preview guidance,
then it says `Enter runs typed: mission mission-42` instead of echoing the alias
token.

### AC-3 Existing canonical slash-prefixed typed preview guidance does not regress
Given earlier slash-prefixed canonical command preview behavior,
when typed alias preview canonicalization lands,
then existing canonical typed preview coverage stays green.

## Conformance Cases
- C-01 Rendering the command palette for `/rs mission-42` shows
  `Enter runs typed: resume mission-42`. Maps to AC-1. Tier: Functional.
- C-02 Rendering the command palette for `/mi mission-42` shows
  `Enter runs typed: mission mission-42`. Maps to AC-2. Tier: Functional.
- C-03 Existing slash-prefixed canonical typed preview coverage still passes
  after the alias canonicalization change. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Slash-prefixed typed alias submissions preview the same canonical command that
  the execution path resolves
- Operators no longer see alias-token drift between palette guidance and Enter
  behavior
- Existing slash-prefixed canonical preview behavior remains green

## Key Decisions
- Canonicalize the first token in typed Enter guidance through the same command
  lookup path used by execution, not by ad hoc string rewriting
