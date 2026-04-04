# Spec: Issue #3721 - Normalize slash-prefixed explicit command-palette submissions in tau-tui

Status: Implemented

## Problem Statement
The `tau-tui` command palette already lets operators type complete command text
directly and preview that Enter will run it, but slash-prefixed typed
submissions with argument payloads like `/search apples` and `/copy-last
assistant` still reach command execution with the leading slash intact. Because
palette execution only canonicalizes unprefixed command tokens, these operator
habits degrade into `Unknown command` instead of behaving like normal
slash-command input.

## Scope
In scope:
- slash-prefixed typed command-palette submissions
- canonical command execution after leading-slash normalization
- argument payload preservation after leading-slash normalization
- focused RED/GREEN gateway coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3721/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command filtering or ranking changes
- preview wording changes
- scaffold or placeholder behavior changes
- new commands or new aliases

## Acceptance Criteria
### AC-1 Slash-prefixed typed commands with arguments execute canonically
Given the operator types a slash-prefixed command with arguments directly into
the command palette,
when they press Enter,
then Tau normalizes the command token and executes the canonical command instead
of surfacing `Unknown command`.

### AC-2 Slash-prefixed typed commands preserve their argument payload
Given the operator types a slash-prefixed command with arguments directly into
the command palette,
when they press Enter,
then Tau strips only the command-token slash and preserves the remaining
arguments so the intended command behavior still runs instead of surfacing
`Unknown command`.

### AC-3 Existing unprefixed typed palette behavior does not regress
Given earlier typed command-palette execution behavior,
when leading-slash normalization lands,
then existing unprefixed typed submissions still execute through the same path.

## Conformance Cases
- C-01 Typing `/search apples` in the command palette executes the transcript
  search flow and surfaces `Search results for "apples"` instead of
  `Unknown command`. Maps to AC-1. Tier: Functional.
- C-02 Typing `/copy-last assistant` in the command palette copies the latest
  assistant transcript entry instead of surfacing `Unknown command`. Maps to
  AC-2. Tier: Functional.
- C-03 Existing unprefixed typed-command palette coverage still passes after
  slash normalization. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can type familiar slash commands into the palette without execution
  surprises
- Slash-prefixed typed commands preserve their intended argument payload
- Earlier typed palette execution coverage remains green

## Key Decisions
- Normalize only the first command token, not the full input payload
- Reuse the existing palette command lookup and alias resolution paths after
  stripping a single leading slash
