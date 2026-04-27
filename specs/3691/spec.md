# Spec: Issue #3691 - Add placeholder-jump editing ergonomics to tau-tui command palette

Status: Accepted

## Problem Statement
The tau-tui command palette currently stores its input as a plain string and
appends or removes characters only at the end. That keeps simple commands
working, but it makes argument-heavy commands awkward because operators cannot
move predictably between placeholder regions after inserting a command template.

Issue #3691 asks for cursor-aware placeholder jump ergonomics so command
palette editing can keep pace with autocomplete and guardrail flows.

## Scope
In scope:
- cursor position tracking for the command palette input
- key handling for moving the command-palette cursor within the input
- a placeholder jump operation that moves to the next placeholder span
- tests proving placeholder jumps are predictable and wrap through multiple
  placeholder spans
- preserving existing command execution behavior for typed commands

Out of scope:
- a full command-template/autocomplete redesign
- visual styling changes outside the cursor position needed for editing
- new gateway commands or command semantics
- persistence of command history
- mouse editing in the command palette

## Acceptance Criteria
### AC-1 Command palette input is cursor-aware
Given the command palette is focused,
when an operator types, moves left or right, and backspaces,
then edits occur at the current command-palette cursor rather than only at the
end of the input.

### AC-2 Placeholder jump moves to the next placeholder region
Given the command palette contains a command template with placeholder spans,
when the operator invokes the placeholder jump key,
then the command-palette cursor moves to the start of the next placeholder span
after the current cursor position.

### AC-3 Placeholder jump wraps predictably
Given the command palette contains multiple placeholder spans,
when the operator invokes placeholder jump after the last placeholder span,
then the command-palette cursor wraps to the first placeholder span.

### AC-4 Existing command execution remains stable
Given an operator submits an existing command such as `help`, `tools`,
`mission <mission-id>`, or `resume <mission-id>`,
when the command palette handles Enter,
then the command is executed with the same command string as before and the
palette clears back to input focus.

## Conformance Cases
- C-01 A unit test proves command-palette insertion/backspace respects an
  interior cursor. Maps to AC-1. Tier: Unit.
- C-02 A unit test proves `placeholder_jump` moves from before the first
  placeholder to that placeholder start. Maps to AC-2. Tier: Unit.
- C-03 A unit test proves repeated placeholder jumps advance through spans and
  wrap to the first span. Maps to AC-3. Tier: Unit.
- C-04 A regression test proves submitting a typed existing command still
  changes app state as before. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- `cargo test -p tau-tui placeholder_jump --lib` passes.
- The command palette has explicit cursor-state code rather than a plain
  append-only string path.
- Root Cargo manifests and lockfiles remain unchanged.