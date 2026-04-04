# Spec: Issue #3704 - Preserve active placeholder focus across Ctrl+U/Ctrl+K in `tau-tui` command palette

Status: Implemented

## Problem Statement
The `tau-tui` command palette now treats scaffold placeholders as atomic spans
for movement and deletion, but line-clearing still falls back to raw cursor
semantics. When an active placeholder is present, `Ctrl+U` and `Ctrl+K` drop
placeholder focus and leave literal `<...>` tokens behind. That forces the
operator back into manual cleanup instead of preserving the current scaffold
field as the editable unit.

## Scope
In scope:
- active-placeholder `Ctrl+U` focus preservation
- active-placeholder `Ctrl+K` focus preservation
- preserving non-placeholder line-clearing behavior
- focused RED/GREEN coverage in `tau-tui`
- spec/plan/tasks updates under `specs/3704/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- new command-palette commands
- runtime/gateway changes
- non-placeholder line-clearing behavior changes
- placeholder movement or delete semantics beyond focus preservation

## Acceptance Criteria
### AC-1 Ctrl+U preserves active placeholder focus after clearing the prefix
Given a scaffold placeholder is active in the command palette,
when the operator presses `Ctrl+U`,
then Tau clears the prefix to the start of the line but keeps the active
placeholder focused so typing still replaces that scaffold field directly.

### AC-2 Ctrl+K preserves active placeholder focus after clearing the suffix
Given a scaffold placeholder is active in the command palette,
when the operator presses `Ctrl+K`,
then Tau clears the suffix to the end of the line but keeps the active
placeholder focused so the current scaffold field remains directly editable.

### AC-3 Existing placeholder and line-clearing flows do not regress
Given existing command-palette placeholder focus and line-clearing behavior,
when placeholder-aware line clearing lands,
then scoped `tau-tui` regression coverage still passes.

## Conformance Cases
- C-01 `Ctrl+U` from an active first placeholder in `copy <source> <dest>`
  preserves active placeholder editing so typing replaces `<source>` directly
  after the prefix is cleared. Maps to AC-1. Tier: Functional.
- C-02 `Ctrl+K` from an active first placeholder in `copy <source> <dest>`
  preserves active placeholder feedback after the suffix is cleared. Maps to
  AC-2. Tier: Functional.
- C-03 Existing command-palette placeholder and line-clearing coverage still
  passes. Maps to AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- Operators can use line-clearing shortcuts around scaffold placeholders without
  dropping back into raw `<...>` token editing
- Placeholder-aware line clearing feels consistent with the rest of the palette
  editor
- Existing command-palette editing flows remain stable

## Key Decisions
- Keep the active placeholder as the current atomic field for `Ctrl+U` and
  `Ctrl+K`
- Preserve ordinary non-placeholder line-clearing behavior
- Limit the slice to focus preservation rather than broader command-line parsing
