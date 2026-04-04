# Spec: Issue #3723 - Surface slash-prefixed typed command matches in tau-tui command palette

Status: Implemented

## Problem Statement
After `#3721` and `#3722`, slash-prefixed typed command-palette inputs execute
and preview canonically, but argument-carrying inputs like `/search apples` and
`/resume mission-42` still fall into the no-match state. That means the palette
hides the underlying command match even when the first token already names a
known slash command.

## Scope
In scope:
- slash-prefixed typed command-palette matching when arguments are present
- canonical command-match surfacing for slash-prefixed typed inputs
- focused RED/GREEN gateway coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3723/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- command execution changes
- Enter-preview wording changes unrelated to matching
- placeholder blocking changes
- new commands or aliases

## Acceptance Criteria
### AC-1 Slash-prefixed typed search input still surfaces the search command
Given the operator types `/search apples` into the command palette,
when Tau evaluates command matches,
then it still surfaces the `search` command instead of falling into the no-match
state.

### AC-2 Slash-prefixed typed resume input still surfaces the resume preview
Given the operator types `/resume mission-42` into the command palette,
when Tau renders the palette,
then it shows the `resume` command preview rather than `No matching commands`.

### AC-3 Existing non-slash matching behavior does not regress
Given earlier command-palette matching behavior,
when slash-prefixed typed command matching lands,
then existing non-slash queries keep their current matching behavior.

## Conformance Cases
- C-01 `command_palette_matches(\"/search apples\")` contains `search`. Maps to
  AC-1. Tier: Functional.
- C-02 Rendering the command palette for `/resume mission-42` shows
  `Preview: resume` and omits `No matching commands`. Maps to AC-2. Tier:
  Functional.
- C-03 Existing non-slash matching coverage still passes after the slash-aware
  matching change. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Slash-prefixed typed commands with argument payloads no longer hide their
  underlying command match
- Operators can keep visual command context while entering slash-prefixed
  commands with real arguments
- Earlier non-slash matching behavior remains green

## Key Decisions
- Match slash-prefixed typed inputs by the canonical first command token when
  present, while preserving the full typed payload for execution/preview
