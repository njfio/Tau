# M325 - Interactive timeout marker contract fix

Status: Active

## Context
Interactive progress output currently shows `interactive.turn=start timeout_ms=0`
in sessions where turn timeout is disabled but request timeout is active, which
is misleading for operators diagnosing wait behavior.

## Issue Hierarchy
- Task: #3558

## Scope
- Make start marker explicitly include both timeout domains:
  - `turn_timeout_ms`
  - `request_timeout_ms`
- Ensure runtime wiring passes request timeout into interactive progress line.
- Update tests/docs to reflect the new marker contract.

## Exit Criteria
- Start marker reports both timeout fields.
- Tests covering marker contract and runtime wiring pass.
- README/operator guide examples match actual emitted lines.
